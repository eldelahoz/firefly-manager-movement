//! Settings use cases (RF-19): get/save with token masking, verify.

use std::sync::Arc;

use async_trait::async_trait;

use ffm_application::dtos::SettingsView;
use ffm_application::dtos::{
    About, Ff3AccountDto, Ff3TransactionDto, NewAppSettings, NewFf3TransactionCreation,
};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{
    AppSettingsRepository, FireflyClient, FireflyClientFactory, FireflyConnectionManager,
};
use ffm_application::use_cases::app_settings::{
    EnvDefaults, GetAppSettings, SaveAppSettings, SaveAppSettingsInput, VerifyFireflyConnection,
};
use ffm_domain::entities::AppSettings;

fn env(url: &str, token: &str) -> EnvDefaults {
    EnvDefaults {
        ff3_url: url.into(),
        ff3_token: token.into(),
    }
}

// ── Fakes ───────────────────────────────────────────────────────────────

struct FakeSettings(std::sync::Mutex<Option<AppSettings>>);

#[async_trait]
impl AppSettingsRepository for FakeSettings {
    async fn get(&self) -> Result<Option<AppSettings>, ApplicationError> {
        Ok(self.0.lock().unwrap().clone())
    }
    async fn save(&self, s: NewAppSettings) -> Result<AppSettings, ApplicationError> {
        let row = AppSettings {
            ff3_url: s.ff3_url,
            ff3_token: s.ff3_token,
            default_creation_tag: s.default_creation_tag,
            enable_origin_trace: s.enable_origin_trace,
            default_date_format: s.default_date_format.and_then(|f| f.parse().ok()),
            updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
        };
        *self.0.lock().unwrap() = Some(row.clone());
        Ok(row)
    }
}

struct FakeManager(std::sync::Arc<std::sync::Mutex<Vec<(String, String)>>>);

impl Default for FakeManager {
    fn default() -> Self {
        Self(std::sync::Arc::new(std::sync::Mutex::new(Vec::new())))
    }
}

impl Clone for FakeManager {
    fn clone(&self) -> Self {
        Self(std::sync::Arc::clone(&self.0))
    }
}

#[async_trait]
impl FireflyConnectionManager for FakeManager {
    async fn reconfigure(&self, url: &str, token: &str) -> Result<(), ApplicationError> {
        self.0
            .lock()
            .unwrap()
            .push((url.to_string(), token.to_string()));
        Ok(())
    }
}

struct FakeProbe;

#[async_trait]
impl FireflyClient for FakeProbe {
    async fn about(&self) -> Result<About, ApplicationError> {
        Ok(About {
            version: "6.1.36".into(),
        })
    }
    async fn asset_accounts(&self) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        Ok(vec![])
    }
    async fn account_transactions(
        &self,
        _account_ff3_id: i64,
        _start: Option<chrono::NaiveDate>,
        _end: Option<chrono::NaiveDate>,
    ) -> Result<Vec<Ff3TransactionDto>, ApplicationError> {
        Ok(vec![])
    }
    async fn create_transaction(
        &self,
        _creation: NewFf3TransactionCreation,
    ) -> Result<i64, ApplicationError> {
        Ok(1)
    }
    async fn delete_transaction(&self, _ff3_id: i64) -> Result<(), ApplicationError> {
        Ok(())
    }
}

struct FakeFactory;

impl FireflyClientFactory for FakeFactory {
    fn build(&self, _url: &str, _token: &str) -> Arc<dyn FireflyClient> {
        Arc::new(FakeProbe)
    }
}

fn epoch() -> AppSettings {
    AppSettings {
        ff3_url: "http://localhost:8080".into(),
        ff3_token: "tok".into(),
        default_creation_tag: None,
        enable_origin_trace: true,
        default_date_format: None,
        updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
    }
}

// ── GetAppSettings ──────────────────────────────────────────────────────

#[tokio::test]
async fn get_returns_empty_view_when_no_row() {
    let repo = FakeSettings(std::sync::Mutex::new(None));
    let view: SettingsView = GetAppSettings::new(repo, env("", ""))
        .execute()
        .await
        .unwrap();
    assert_eq!(view.ff3_url, "");
    assert!(!view.has_token);
    assert_eq!(view.default_creation_tag, None);
}

#[tokio::test]
async fn get_masks_token_but_reports_presence() {
    let repo = FakeSettings(std::sync::Mutex::new(Some(epoch())));
    let view = GetAppSettings::new(repo, env("", ""))
        .execute()
        .await
        .unwrap();
    assert_eq!(view.ff3_url, "http://localhost:8080");
    assert!(view.has_token);
    // SettingsView carries no token field at all (compile-time guarantee, NFR-02).
}

#[tokio::test]
async fn get_reflects_env_fallback_when_no_row() {
    let repo = FakeSettings(std::sync::Mutex::new(None));
    let view = GetAppSettings::new(repo, env("http://env-ff3:8080", "envtok"))
        .execute()
        .await
        .unwrap();
    assert_eq!(view.ff3_url, "http://env-ff3:8080");
    assert!(view.has_token);
    assert_eq!(view.default_creation_tag, None);
}

// ── SaveAppSettings ─────────────────────────────────────────────────────

#[tokio::test]
async fn save_persists_reconfigures_and_keeps_token_when_omitted() {
    let repo = FakeSettings(std::sync::Mutex::new(Some(epoch())));
    let mgr = FakeManager::default();
    let view = SaveAppSettings::new(repo, mgr.clone(), env("", ""))
        .execute(SaveAppSettingsInput {
            ff3_url: "http://ff3.new".into(),
            ff3_token: None, // keep existing
            default_creation_tag: Some("ffm".into()),
            enable_origin_trace: true,
            default_date_format: None,
        })
        .await
        .unwrap();
    assert_eq!(view.ff3_url, "http://ff3.new");
    assert_eq!(view.default_creation_tag.as_deref(), Some("ffm"));
    assert_eq!(
        mgr.0.lock().unwrap()[0],
        ("http://ff3.new".to_string(), "tok".to_string())
    );
}

#[tokio::test]
async fn save_replaces_token_when_provided_and_clears_tag() {
    let repo = FakeSettings(std::sync::Mutex::new(Some(epoch())));
    let mgr = FakeManager::default();
    let view = SaveAppSettings::new(repo, mgr, env("", ""))
        .execute(SaveAppSettingsInput {
            ff3_url: "http://ff3.new".into(),
            ff3_token: Some("  tok2 ".into()),
            default_creation_tag: Some("   ".into()), // blank clears
            enable_origin_trace: true,
            default_date_format: None,
        })
        .await
        .unwrap();
    assert!(view.has_token);
    assert_eq!(view.default_creation_tag, None);
}

#[tokio::test]
async fn save_rejects_blank_url() {
    let repo = FakeSettings(std::sync::Mutex::new(Some(epoch())));
    let err = SaveAppSettings::new(repo, FakeManager::default(), env("", ""))
        .execute(SaveAppSettingsInput {
            ff3_url: "  ".into(),
            ff3_token: Some("t".into()),
            default_creation_tag: None,
            enable_origin_trace: true,
            default_date_format: None,
        })
        .await
        .unwrap_err();
    assert!(matches!(err, ApplicationError::Parse(_)));
}

#[tokio::test]
async fn save_rejects_non_http_url() {
    let repo = FakeSettings(std::sync::Mutex::new(Some(epoch())));
    let err = SaveAppSettings::new(repo, FakeManager::default(), env("", ""))
        .execute(SaveAppSettingsInput {
            ff3_url: "ftp://ff3".into(),
            ff3_token: Some("t".into()),
            default_creation_tag: None,
            enable_origin_trace: true,
            default_date_format: None,
        })
        .await
        .unwrap_err();
    assert!(matches!(err, ApplicationError::Parse(_)));
}

#[tokio::test]
async fn save_without_any_token_is_rejected() {
    let repo = FakeSettings(std::sync::Mutex::new(None)); // no stored token
    let err = SaveAppSettings::new(repo, FakeManager::default(), env("", ""))
        .execute(SaveAppSettingsInput {
            ff3_url: "http://x".into(),
            ff3_token: None,
            default_creation_tag: None,
            enable_origin_trace: true,
            default_date_format: None,
        })
        .await
        .unwrap_err();
    assert!(matches!(err, ApplicationError::Config(_)));
}

#[tokio::test]
async fn save_bootstraps_token_from_env_when_no_row() {
    let repo = std::sync::Arc::new(FakeSettings(std::sync::Mutex::new(None)));
    let mgr = FakeManager::default();
    let view = SaveAppSettings::new(
        repo.clone(),
        mgr.clone(),
        env("http://env-ff3:8080", "envtok"),
    )
    .execute(SaveAppSettingsInput {
        ff3_url: "http://ff3.new".into(),
        ff3_token: None, // no row to keep from: fall back to env token
        default_creation_tag: None,
        enable_origin_trace: true,
        default_date_format: None,
    })
    .await
    .unwrap();
    assert!(view.has_token);
    // Persisted row now carries the bootstrapped token.
    let stored = repo.get().await.unwrap().expect("row saved");
    assert_eq!(stored.ff3_token, "envtok");
    assert_eq!(
        mgr.0.lock().unwrap()[0],
        ("http://ff3.new".to_string(), "envtok".to_string())
    );
}

// ── VerifyFireflyConnection ─────────────────────────────────────────────

#[tokio::test]
async fn verify_probes_via_factory() {
    let res = VerifyFireflyConnection::new(FakeFactory)
        .execute("http://ff3".into(), "tok".into())
        .await
        .unwrap();
    assert_eq!(res.version, "6.1.36");
}

#[tokio::test]
async fn verify_rejects_invalid_url() {
    let err = VerifyFireflyConnection::new(FakeFactory)
        .execute("not-a-url".into(), "tok".into())
        .await
        .unwrap_err();
    assert!(matches!(err, ApplicationError::Parse(_)));
}

#[tokio::test]
async fn verify_rejects_blank_token() {
    let err = VerifyFireflyConnection::new(FakeFactory)
        .execute("http://ff3".into(), " ".into())
        .await
        .unwrap_err();
    assert!(matches!(err, ApplicationError::Parse(_)));
}

// #region Default date format (RF-02 v0.8)

#[tokio::test]
async fn save_persists_default_date_format() {
    let repo = FakeSettings(std::sync::Mutex::new(Some(epoch())));
    let view = SaveAppSettings::new(repo, FakeManager::default(), env("", ""))
        .execute(SaveAppSettingsInput {
            ff3_url: "http://ff3.new".into(),
            ff3_token: None,
            default_creation_tag: None,
            enable_origin_trace: true,
            default_date_format: Some("dd/mm/yyyy".into()),
        })
        .await
        .unwrap();
    assert_eq!(view.default_date_format.as_deref(), Some("dd/mm/yyyy"));
}

#[tokio::test]
async fn get_returns_stored_default_date_format() {
    let repo = std::sync::Arc::new(FakeSettings(std::sync::Mutex::new(Some(epoch()))));
    SaveAppSettings::new(repo.clone(), FakeManager::default(), env("", ""))
        .execute(SaveAppSettingsInput {
            ff3_url: "http://ff3.new".into(),
            ff3_token: None,
            default_creation_tag: None,
            enable_origin_trace: true,
            default_date_format: Some("mm/dd/yyyy".into()),
        })
        .await
        .unwrap();
    let view = GetAppSettings::new(repo, env("", ""))
        .execute()
        .await
        .unwrap();
    assert_eq!(view.default_date_format.as_deref(), Some("mm/dd/yyyy"));
}

#[tokio::test]
async fn save_rejects_unknown_default_date_format() {
    let repo = FakeSettings(std::sync::Mutex::new(Some(epoch())));
    let err = SaveAppSettings::new(repo, FakeManager::default(), env("", ""))
        .execute(SaveAppSettingsInput {
            ff3_url: "http://ff3.new".into(),
            ff3_token: None,
            default_creation_tag: None,
            enable_origin_trace: true,
            default_date_format: Some("bogus".into()),
        })
        .await
        .unwrap_err();
    assert!(matches!(err, ApplicationError::Parse(_)));
}

#[tokio::test]
async fn save_clears_default_date_format_with_none() {
    let repo = FakeSettings(std::sync::Mutex::new(Some(epoch())));
    let view = SaveAppSettings::new(repo, FakeManager::default(), env("", ""))
        .execute(SaveAppSettingsInput {
            ff3_url: "http://ff3.new".into(),
            ff3_token: None,
            default_creation_tag: None,
            enable_origin_trace: true,
            default_date_format: None,
        })
        .await
        .unwrap();
    assert_eq!(view.default_date_format, None);
}
// #endregion
