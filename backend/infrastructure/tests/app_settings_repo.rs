//! app_settings repository: migration reversibility (down -> up) + roundtrip.

mod common;

use diesel_async::SimpleAsyncConnection;
use ffm_application::dtos::NewAppSettings;
use ffm_application::ports::AppSettingsRepository;
use ffm_infrastructure::persistence::repositories::app_settings::MysqlAppSettingsRepository;

const UP_SQL: &str = include_str!("../migrations/2026-09-03-100000-0006_app_settings/up.sql");
const DOWN_SQL: &str = include_str!("../migrations/2026-09-03-100000-0006_app_settings/down.sql");

#[tokio::test]
async fn app_settings_repo_roundtrip_and_reversible_migration() {
    let db = common::setup_pool().await;
    common::clean(&db).await;
    let repo = MysqlAppSettingsRepository::new(db.clone());

    // empty by default
    assert!(repo.get().await.unwrap().is_none());

    // save + read back (token present backend-side only)
    let saved = repo
        .save(NewAppSettings {
            ff3_url: "http://localhost:8080".into(),
            ff3_token: "secret-token".into(),
            default_creation_tag: Some("ffm-import".into()),
        })
        .await
        .unwrap();
    assert_eq!(saved.ff3_url, "http://localhost:8080");
    assert_eq!(saved.default_creation_tag.as_deref(), Some("ffm-import"));
    let loaded = repo.get().await.unwrap().unwrap();
    assert_eq!(loaded.ff3_token, "secret-token");

    // clearing the tag round-trips to None
    repo.save(NewAppSettings {
        ff3_url: "http://localhost:8080".into(),
        ff3_token: "secret-token".into(),
        default_creation_tag: None,
    })
    .await
    .unwrap();
    assert!(repo
        .get()
        .await
        .unwrap()
        .unwrap()
        .default_creation_tag
        .is_none());

    // migration is reversible: down then up, then the table still works.
    // DDL must go through the text protocol (batch_execute), not the
    // prepared-statement protocol.
    {
        let mut conn = db.get().await.unwrap();
        conn.batch_execute(DOWN_SQL).await.unwrap();
        conn.batch_execute(UP_SQL).await.unwrap();
    }
    assert!(repo.get().await.unwrap().is_none());
}
