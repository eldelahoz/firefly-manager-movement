mod common;

use ffm_application::dtos::NewDeletedAudit;
use ffm_application::ports::DeletedAuditRepository;
use ffm_infrastructure::persistence::repositories::deleted_audit::MysqlDeletedAuditRepository;

#[tokio::test]
async fn insert_returns_increasing_ids() {
    let pool = common::setup_pool().await;
    common::clean(&pool).await;
    let repo = MysqlDeletedAuditRepository::new(pool.clone());

    let id1 = repo
        .insert(NewDeletedAudit {
            ff3_id: 42,
            snapshot_json: r#"{"description":"x"}"#.into(),
        })
        .await
        .expect("insert 1");
    let id2 = repo
        .insert(NewDeletedAudit {
            ff3_id: 43,
            snapshot_json: r#"{"description":"y"}"#.into(),
        })
        .await
        .expect("insert 2");

    assert!(id1.as_i64() > 0);
    assert!(id2.as_i64() > id1.as_i64());

    let ids = repo.deleted_ff3_ids().await.expect("deleted_ff3_ids");
    assert!(ids.contains(&42) && ids.contains(&43));

    common::clean(&pool).await;
}
