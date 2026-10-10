use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use ffm_application::dtos::{About, Ff3AccountDto, Ff3TransactionDto, NewFf3TransactionCreation};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{FireflyClient, FireflyClientFactory, FireflyConnectionManager};
use ffm_domain::entities::AppSettings;
use ffm_presentation::{app, AppState};
use serde_json::json;
use tower::ServiceExt;

mod common;

use common::FakeAppSettingsRepository;

// ── Fakes ───────────────────────────────────────────────────────────────

#[derive(Clone)]
struct FakeSettingsManager(Arc<std::sync::Mutex<Vec<(String, String)>>>);

impl Default for FakeSettingsManager {
    fn default() -> Self {
        Self(Arc::new(std::sync::Mutex::new(Vec::new())))
    }
}

#[async_trait::async_trait]
impl FireflyConnectionManager for FakeSettingsManager {
    async fn reconfigure(&self, url: &str, token: &str) -> Result<(), ApplicationError> {.
        if url == "http://slow" {
            tokio::time::sleep(std::time::Duration::from_millis(80)).await;
        }
        self.0
            .lock()
            .unwrap()
            .push((url.to_string(), token.to_string()));
        Ok(())
    }
}

struct FakeVerifyClient {
    fail: bool,
}

#[async_trait::async_trait]
impl FireflyClient for FakeVerifyClient {
    async fn about(&self) -> Result<About, ApplicationError> {
        if self.fail {
            Err(ApplicationError::FireflyClient("unreachable".into()))
        } else {
            Ok(About {
                version: "6.1.36".into(),
            })
        }
    }
    async fn asset_accounts(&self) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        Ok(vec![])
    }
    async fn account_transactions(
        &self,
        _id: i64,
        _start: Option<chrono::NaiveDate>,
        _end: Option<chrono::NaiveDate>,
    ) -> Result<Vec<Ff3TransactionDto>, ApplicationError> {
        Ok(vec![])
    }
    async fn create_transaction(
        &self,
        _c: NewFf3TransactionCreation,
    ) -> Result<i64, ApplicationError> {
        Ok(1)
    }
    async fn delete_transaction(&self, _id: i64) -> Result<(), ApplicationError> {
        Ok(())
    }
}

struct FakeVerifyFactory {
    fail: bool,
}

impl FireflyClientFactory for FakeVerifyFactory {
    fn build(&self, _url: &str, _token: &str) -> Arc<dyn FireflyClient> {
        Arc::new(FakeVerifyClient { fail: self.fail })
    }
}

fn make_state(
    settings: FakeAppSettingsRepository,
    manager: FakeSettingsManager,
    factory: FakeVerifyFactory,
) -> AppState {
    make_state_with_env(settings, manager, factory, Default::default())
}

fn make_state_with_env(
    settings: FakeAppSettingsRepository,
    manager: FakeSettingsManager,
    factory: FakeVerifyFactory,
    env_defaults: ffm_application::use_cases::app_settings::EnvDefaults,
) -> AppState {
    AppState {
        firefly: Arc::new(common::FakeFireflyClient::ok()),
        ff3_accounts: Arc::new(common::FakeFf3AccountRepository::new()),
        csv_decoder: Arc::new(common::FakeCsvDecoder),
        batches: Arc::new(common::FakeBatchRepository),
        movements: Arc::new(common::FakeMovementRepository),
        csv_mappings: Arc::new(common::FakeCsvMappingRepository),
        reconciliations: Arc::new(common::FakeReconciliationStore::new()),
        matches: Arc::new(common::FakeReconciliationStore::new()),
        ff3_transactions: Arc::new(common::FakeReconciliationStore::new()),
        creations: Arc::new(common::FakeCreationRepository),
        deleted_audits: Arc::new(common::FakeDeletedAuditRepository::default()),
        ff3_health_probe: false,
        app_settings: Arc::new(settings),
        firefly_manager: Arc::new(manager),
        firefly_factory: Arc::new(factory),
        env_defaults,
        settings_lock: Arc::new(tokio::sync::Mutex::new(())),
    }
}

// ── GET /api/settings ──────────────────────────────────────────────────

#[tokio::test]
async fn get_settings_empty_when_no_row() {
    let state = make_state(
        FakeAppSettingsRepository::new(),
        FakeSettingsManager::default(),
        FakeVerifyFactory { fail: false },
    );
    let router = app(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["ff3_url"], "");
    assert_eq!(body["has_token"], false);
    assert_eq!(body["default_creation_tag"], serde_json::Value::Null);
    // Token field must never appear (NFR-02 compile-time guarantee).
    assert!(body.get("ff3_token").is_none());
}

#[tokio::test]
async fn get_settings_reflects_env_fallback_when_no_row() {
    let state = make_state_with_env(
        FakeAppSettingsRepository::new(),
        FakeSettingsManager::default(),
        FakeVerifyFactory { fail: false },
        ffm_application::use_cases::app_settings::EnvDefaults {
            ff3_url: "http://env-ff3:8080".into(),
            ff3_token: "envtok".into(),
        },
    );
    let router = app(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["ff3_url"], "http://env-ff3:8080");
    assert_eq!(body["has_token"], true);
    assert!(body.get("ff3_token").is_none());
}

#[tokio::test]
async fn get_settings_returns_url_and_has_token() {
    let settings = FakeAppSettingsRepository::with_settings(AppSettings {
        ff3_url: "http://ff3.local".into(),
        ff3_token: "secret".into(),
        default_creation_tag: Some("ffm".into()),
        enable_origin_trace: true,
        default_date_format: None,
        updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
    });
    let state = make_state(
        settings,
        FakeSettingsManager::default(),
        FakeVerifyFactory { fail: false },
    );
    let router = app(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["ff3_url"], "http://ff3.local");
    assert_eq!(body["has_token"], true);
    assert_eq!(body["default_creation_tag"], "ffm");
    assert!(body.get("ff3_token").is_none());
}

// ── PUT /api/settings ──────────────────────────────────────────────────

#[tokio::test]
async fn put_settings_saves_and_reconfigures() {
    let settings = FakeAppSettingsRepository::with_settings(AppSettings {
        ff3_url: "http://old".into(),
        ff3_token: "old_tok".into(),
        default_creation_tag: None,
        enable_origin_trace: true,
        default_date_format: None,
        updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
    });
    let manager = FakeSettingsManager::default();
    let state = make_state(settings, manager.clone(), FakeVerifyFactory { fail: false });
    let router = app(state);

    let response = router
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/settings")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "ff3_url": "http://ff3.new",
                        "ff3_token": "new_tok",
                        "default_creation_tag": "ffm"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["ff3_url"], "http://ff3.new");
    assert_eq!(body["has_token"], true);
    assert_eq!(body["default_creation_tag"], "ffm");

    // Manager received reconfigure call.
    let calls = manager.0.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0],
        ("http://ff3.new".to_string(), "new_tok".to_string())
    );
}

#[tokio::test]
async fn put_settings_keeps_existing_token_when_omitted() {
    let settings = FakeAppSettingsRepository::with_settings(AppSettings {
        ff3_url: "http://old".into(),
        ff3_token: "keep_me".into(),
        default_creation_tag: None,
        enable_origin_trace: true,
        default_date_format: None,
        updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
    });
    let manager = FakeSettingsManager::default();
    let state = make_state(settings, manager.clone(), FakeVerifyFactory { fail: false });
    let router = app(state);

    let response = router
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/settings")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "ff3_url": "http://ff3.new"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["has_token"], true);

    let calls = manager.0.lock().unwrap();
    assert_eq!(calls[0].1, "keep_me");
}

#[tokio::test]
async fn concurrent_saves_keep_reconfigure_in_sync_with_storage() {
    let settings = FakeAppSettingsRepository::with_settings(AppSettings {
        ff3_url: "http://old".into(),
        ff3_token: "old_tok".into(),
        default_creation_tag: None,
        enable_origin_trace: true,
        default_date_format: None,
        updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
    });
    let manager = FakeSettingsManager::default();
    let state = make_state(settings, manager.clone(), FakeVerifyFactory { fail: false });

    let put = |url: &str| {
        Request::builder()
            .method("PUT")
            .uri("/api/settings")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "ff3_url": url, "ff3_token": "tok" }).to_string(),
            ))
            .unwrap()
    };

    // "slow" writes first but records its reconfigure last; "lag" writes last
    // and records immediately. Unserialized, the LAST reconfigure would match
    // neither the persisted row (fast inversion at ~40ms vs ~80ms).
    let (ra, rb) = tokio::join!(
        app(state.clone()).oneshot(put("http://slow")),
        app(state.clone()).oneshot(put("http://lag"))
    );
    assert_eq!(ra.unwrap().status(), StatusCode::OK);
    assert_eq!(rb.unwrap().status(), StatusCode::OK);

    let response = app(state)
        .oneshot(
            Request::builder()
                .uri("/api/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let stored: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let calls = manager.0.lock().unwrap();
    assert_eq!(calls.len(), 2);
    assert_eq!(stored["ff3_url"].as_str().unwrap(), calls.last().unwrap().0);
}

#[tokio::test]
async fn put_settings_blank_url_returns_400() {
    let state = make_state(
        FakeAppSettingsRepository::new(),
        FakeSettingsManager::default(),
        FakeVerifyFactory { fail: false },
    );
    let router = app(state);

    let response = router
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/settings")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "ff3_url": "  ",
                        "ff3_token": "tok"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ── POST /api/settings/verify ──────────────────────────────────────────

#[tokio::test]
async fn verify_ok_returns_version() {
    let state = make_state(
        FakeAppSettingsRepository::new(),
        FakeSettingsManager::default(),
        FakeVerifyFactory { fail: false },
    );
    let router = app(state);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/settings/verify")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "ff3_url": "http://ff3",
                        "ff3_token": "tok"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["version"], "6.1.36");
}

#[tokio::test]
async fn verify_unreachable_returns_502() {
    let state = make_state(
        FakeAppSettingsRepository::new(),
        FakeSettingsManager::default(),
        FakeVerifyFactory { fail: true },
    );
    let router = app(state);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/settings/verify")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "ff3_url": "http://ff3",
                        "ff3_token": "tok"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn verify_bad_url_returns_400() {
    let state = make_state(
        FakeAppSettingsRepository::new(),
        FakeSettingsManager::default(),
        FakeVerifyFactory { fail: false },
    );
    let router = app(state);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/settings/verify")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "ff3_url": "not-a-url",
                        "ff3_token": "tok"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
