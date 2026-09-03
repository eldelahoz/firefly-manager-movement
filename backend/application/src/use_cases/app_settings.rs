//! App settings use cases (RF-19): read (token masked), save (with
//! connection reconfiguration) and connectivity check.

use crate::dtos::SettingsView;
use crate::dtos::{About, NewAppSettings};
use crate::error::ApplicationError;
use crate::ports::{
    AppSettingsRepository, FireflyClientFactory, FireflyConnectionManager,
};

pub struct GetAppSettings<R> {
    repo: R,
}

impl<R> GetAppSettings<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: AppSettingsRepository> GetAppSettings<R> {
    pub async fn execute(&self) -> Result<SettingsView, ApplicationError> {
        Ok(match self.repo.get().await? {
            Some(s) => SettingsView {
                ff3_url: s.ff3_url,
                has_token: true,
                default_creation_tag: s.default_creation_tag,
            },
            None => SettingsView {
                ff3_url: String::new(),
                has_token: false,
                default_creation_tag: None,
            },
        })
    }
}

pub struct SaveAppSettingsInput {
    pub ff3_url: String,
    /// None/empty = keep the existing token.
    pub ff3_token: Option<String>,
    pub default_creation_tag: Option<String>,
}

pub struct SaveAppSettings<R, M> {
    repo: R,
    manager: M,
}

impl<R, M> SaveAppSettings<R, M> {
    pub fn new(repo: R, manager: M) -> Self {
        Self { repo, manager }
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
        let token = input
            .ff3_token
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .or_else(|| current.as_ref().map(|c| c.ff3_token.clone()))
            .filter(|t| !t.is_empty())
            .ok_or_else(|| ApplicationError::Config("an FF3 API token is required".into()))?;
        let tag = input
            .default_creation_tag
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty());
        self.repo
            .save(NewAppSettings {
                ff3_url: url.clone(),
                ff3_token: token.clone(),
                default_creation_tag: tag.clone(),
            })
            .await?;
        self.manager.reconfigure(&url, &token).await?;
        Ok(SettingsView {
            ff3_url: url,
            has_token: true,
            default_creation_tag: tag,
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
