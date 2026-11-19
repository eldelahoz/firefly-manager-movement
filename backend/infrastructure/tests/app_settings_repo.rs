//! app_settings repository: migration reversibility (down -> up) + roundtrip.

mod common;

use diesel_async::AsyncMigrationHarness;
use diesel_migrations::MigrationHarness;
use ffm_application::dtos::NewAppSettings;
use ffm_application::ports::AppSettingsRepository;
use ffm_domain::value_objects::DateFormat;
use ffm_infrastructure::persistence::migrations::MIGRATIONS;
use ffm_infrastructure::persistence::repositories::app_settings::SqliteAppSettingsRepository;

#[tokio::test(flavor = "multi_thread")] // AsyncMigrationHarness blocks in place
async fn app_settings_repo_roundtrip_and_reversible_migration() {
    let db = common::setup_pool().await;
    let repo = SqliteAppSettingsRepository::new(db.clone());

    // empty by default
    assert!(repo.get().await.unwrap().is_none());

    // save + read back (token present backend-side only)
    let saved = repo
        .save(NewAppSettings {
            ff3_url: "http://localhost:8080".into(),
            ff3_token: "secret-token".into(),
            default_creation_tag: Some("ffm-import".into()),
            enable_origin_trace: true,
            default_date_format: Some("mm/dd/yyyy".into()),
        })
        .await
        .unwrap();
    assert_eq!(saved.ff3_url, "http://localhost:8080");
    assert_eq!(saved.default_creation_tag.as_deref(), Some("ffm-import"));
    let loaded = repo.get().await.unwrap().unwrap();
    assert_eq!(loaded.ff3_token, "secret-token");
    assert_eq!(loaded.default_date_format, Some(DateFormat::MmDdYyyySlash));

    // clearing the tag round-trips to None
    repo.save(NewAppSettings {
        ff3_url: "http://localhost:8080".into(),
        ff3_token: "secret-token".into(),
        default_creation_tag: None,
        enable_origin_trace: true,
        default_date_format: None,
    })
    .await
    .unwrap();
    let loaded = repo.get().await.unwrap().unwrap();
    assert!(loaded.default_creation_tag.is_none());
    assert!(loaded.default_date_format.is_none());

    // Firefly III OAuth personal access tokens are ~1 KB JWTs; the live
    // table must accept them (migration 0008 widened ff3_token to TEXT).
    let long_token = "eyJ".to_string() + &"a".repeat(1025);
    repo.save(NewAppSettings {
        ff3_url: "http://localhost:8080".into(),
        ff3_token: long_token.clone(),
        default_creation_tag: None,
        enable_origin_trace: true,
        default_date_format: None,
    })
    .await
    .expect("save with 1028-char token");
    let loaded = repo.get().await.unwrap().unwrap();
    assert_eq!(loaded.ff3_token, long_token);

    // migrations are reversible: revert all, then re-apply.
    {
        let mut harness = AsyncMigrationHarness::new(db.get().await.unwrap());
        harness.revert_all_migrations(MIGRATIONS).unwrap();
        harness.run_pending_migrations(MIGRATIONS).unwrap();
    }
    assert!(repo.get().await.unwrap().is_none());
}
