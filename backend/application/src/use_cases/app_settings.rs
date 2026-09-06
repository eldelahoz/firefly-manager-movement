//! App settings use cases (RF-19): read (token masked), save (with
//! connection reconfiguration) and connectivity check.

use crate::dtos::SettingsView;
use crate::dtos::{About, NewAppSettings};
use crate::error::ApplicationError;
use crate::ports::{AppSettingsRepository, FireflyClientFactory, FireflyConnectionManager};

/// Effective environment-variable fallback (RF-19, TRD §4): used when no
/// settings row exists yet. Carries the raw env token server-side only —
/// the token is never surfaced to the presentation layer (NFR-02).
#[derive(Clone, Default)]
pub struct EnvDefaults {
    pub ff3_url: String,
    pub ff3_token: String,
}

impl EnvDefaults {
    fn has_token(&self) -> bool {
        !self.ff3_token.is_empty()
    }
}

pub struct GetAppSettings<R> {
    repo: R,
    env: EnvDefaults,
}

impl<R> GetAppSettings<R> {
    pub fn new(repo: R, env: EnvDefaults) -> Self {
        Self { repo, env }
    }
}

impl<R: AppSettingsRepository> GetAppSettings<R> {
    pub async fn execute(&self) -> Result<SettingsView, ApplicationError> {
        Ok(match self.repo.get().await? {
            Some(s) => SettingsView {
                ff3_url: s.ff3_url,
                has_token: true,
                default_creation_tag: s.default_creation_tag,
                enable_origin_trace: s.enable_origin_trace,
            },
            // No row yet: the running config comes from env fallback.
            None => SettingsView {
                ff3_url: self.env.ff3_url.clone(),
                has_token: self.env.has_token(),
                default_creation_tag: None,
                enable_origin_trace: true,
            },
        })
    }
}

pub struct SaveAppSettingsInput {
    pub ff3_url: String,
    /// None/empty = keep the existing token.
    pub ff3_token: Option<String>,
    pub default_creation_tag: Option<String>,
    pub enable_origin_trace: bool,
}

pub struct SaveAppSettings<R, M> {
    repo: R,
    manager: M,
    env: EnvDefaults,
}

impl<R, M> SaveAppSettings<R, M> {
    pub fn new(repo: R, manager: M, env: EnvDefaults) -> Self {
        Self { repo, manager, env }
    }
}

impl<R: AppSettingsRepository, M: FireflyConnectionManager> SaveAppSettings<R, M> {
    pub async fn execute(
        &self,
        input: SaveAppSettingsInput,
    ) -> Result<SettingsView, ApplicationError> {
        let url = input.ff3_url.trim().to_string();
        if url.is_empty() || !(url.starts_with("http://") || url.starts_with("https://")) {
            return Err(ApplicationError::Parse(
                "ff3_url must be a http(s) URL".into(),
            ));
        }
        let current = self.repo.get().await?;
        // Token resolution: input → stored row → env bootstrap (RF-19).
        let token = input
            .ff3_token
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .or_else(|| current.as_ref().map(|c| c.ff3_token.clone()))
            .filter(|t| !t.is_empty())
            .or_else(|| {
                if self.env.has_token() {
                    Some(self.env.ff3_token.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| ApplicationError::Config("an FF3 API token is required".into()))?;
        let tag = input
            .default_creation_tag
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty());
        let enable_origin_trace = input.enable_origin_trace;
        self.repo
            .save(NewAppSettings {
                ff3_url: url.clone(),
                ff3_token: token.clone(),
                default_creation_tag: tag.clone(),
                enable_origin_trace,
            })
            .await?;
        self.manager.reconfigure(&url, &token).await?;
        Ok(SettingsView {
            ff3_url: url,
            has_token: true,
            default_creation_tag: tag,
            enable_origin_trace,
        })
    }
}

pub struct VerifyFireflyConnection<F> {
    factory: F,
}

impl<F> VerifyFireflyConnection<F> {
    pub fn new(factory: F) -> Self {
        Self { factory }
    }
}

impl<F: FireflyClientFactory> VerifyFireflyConnection<F> {
    pub async fn execute(
        &self,
        ff3_url: String,
        ff3_token: String,
    ) -> Result<About, ApplicationError> {
        let url = ff3_url.trim().to_string();
        let token = ff3_token.trim().to_string();
        if url.is_empty() || !(url.starts_with("http://") || url.starts_with("https://")) {
            return Err(ApplicationError::Parse(
                "ff3_url must be a http(s) URL".into(),
            ));
        }
        if token.is_empty() {
            return Err(ApplicationError::Parse(
                "ff3_token must not be empty".into(),
            ));
        }
        let client = self.factory.build(&url, &token);
        client.about().await
    }
}

/// Test-only fake settings repo for integration tests.
#[doc(hidden)]
pub struct FakeSettings(pub std::sync::Mutex<Option<ffm_domain::entities::AppSettings>>);

#[async_trait::async_trait]
impl AppSettingsRepository for FakeSettings {
    async fn get(&self) -> Result<Option<ffm_domain::entities::AppSettings>, ApplicationError> {
        Ok(self.0.lock().unwrap().clone())
    }
    async fn save(
        &self,
        _settings: crate::dtos::NewAppSettings,
    ) -> Result<ffm_domain::entities::AppSettings, ApplicationError> {
        unimplemented!("FakeSettings::save")
    }
}
