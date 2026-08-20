mod common;

use ffm_application::dtos::NewBatch;
use ffm_application::ports::BatchRepository;
use ffm_domain::value_objects::BatchStatus;
use ffm_infrastructure::persistence::repositories::batch::MysqlBatchRepository;

#[tokio::test]
async fn insert_get_list_update_batch() {
    let pool = common::setup_pool().await;
    common::clean(&pool).await;
    let repo = MysqlBatchRepository::new(pool.clone());

    let id = repo
        .insert(NewBatch {
            filename: "test.csv".into(),
            file_hash: "hash123".into(),
            format: "csv".into(),
            currency: "EUR".into(),
            account_id: None,
            status: BatchStatus::Parsed,
        })
        .await
        .expect("insert");

    let got = repo.get(id).await.expect("get");
    assert_eq!(got.id, id);
    assert_eq!(got.filename, "test.csv");
    assert_eq!(got.currency, "EUR");
    assert_eq!(got.status, BatchStatus::Parsed);
    assert!(got.account_id.is_none());

    let list = repo.list().await.expect("list");
    assert_eq!(list.len(), 1);

    repo.update_status(id, BatchStatus::Reconciled)
        .await
        .expect("update_status");
    let updated = repo.get(id).await.expect("get after update");
    assert_eq!(updated.status, BatchStatus::Reconciled);

    common::clean(&pool).await;
}
