mod common;

use diesel_async::RunQueryDsl;

use ffm_application::dtos::{NewBatch, NewReconciliation};
use ffm_application::ports::{BatchRepository, ReconciliationRepository};
use ffm_domain::ids::BatchId;
use ffm_domain::value_objects::BatchStatus;
use ffm_infrastructure::persistence::repositories::batch::MysqlBatchRepository;
use ffm_infrastructure::persistence::repositories::reconciliation::MysqlReconciliationRepository;

async fn insert_batch(pool: &common::DbPool) -> BatchId {
    let batches = MysqlBatchRepository::new(pool.clone());
    batches
        .insert(NewBatch {
            filename: "recon.csv".into(),
            file_hash: "h".into(),
            format: "csv".into(),
            currency: "EUR".into(),
            account_id: None,
            status: BatchStatus::Parsed,
        })
        .await
        .expect("insert batch")
}

fn new_recon(batch_id: BatchId) -> NewReconciliation {
    NewReconciliation {
        batch_id,
        tolerance_days: 2,
        window_start: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        window_end: chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        matched_count: 1,
        missing_count: 2,
        extra_count: 3,
    }
}

#[tokio::test]
async fn insert_and_latest_by_batch() {
    let pool = common::setup_pool().await;
    common::clean(&pool).await;
    let repo = MysqlReconciliationRepository::new(pool.clone());

    // No reconciliations: None, not an error.
    assert!(repo
        .latest_by_batch(BatchId::new(999))
        .await
        .expect("none")
        .is_none());

    let batch_id = insert_batch(&pool).await;
    let id1 = repo.insert(new_recon(batch_id)).await.expect("insert 1");
    let id2 = repo
        .insert(NewReconciliation {
            matched_count: 5,
            ..new_recon(batch_id)
        })
        .await
        .expect("insert 2");

    // ran_at is a DATETIME with second resolution: two inserts in the same
    // second end up tied. We force distinct ran_at values with raw SQL to
    // verify `ORDER BY ran_at DESC` deterministically.
    let mut conn = pool.get().await.expect("conn");
    diesel::sql_query("UPDATE reconciliations SET ran_at = '2026-01-01 00:00:01' WHERE id = ?")
        .bind::<diesel::sql_types::BigInt, _>(id1.as_i64())
        .execute(&mut conn)
        .await
        .expect("set ran_at id1");
    diesel::sql_query("UPDATE reconciliations SET ran_at = '2026-01-01 00:00:02' WHERE id = ?")
        .bind::<diesel::sql_types::BigInt, _>(id2.as_i64())
        .execute(&mut conn)
        .await
        .expect("set ran_at id2");

    let latest = repo
        .latest_by_batch(batch_id)
        .await
        .expect("latest")
        .expect("some");
    assert_eq!(latest.id, id2, "la más reciente por ran_at");
    assert_eq!(latest.matched_count, 5);

    // Another batch with no reconciliations: None.
    assert!(repo
        .latest_by_batch(BatchId::new(1234))
        .await
        .expect("other none")
        .is_none());

    common::clean(&pool).await;
}
