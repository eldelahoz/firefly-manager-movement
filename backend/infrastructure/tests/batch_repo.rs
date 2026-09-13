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
            headers: vec!["Fecha".into(), "Valor".into(), "Desc".into()],
            detected_delimiter: ";".into(),
        })
        .await
        .expect("insert");

    let got = repo.get(id).await.expect("get");
    assert_eq!(got.id, id);
    assert_eq!(got.filename, "test.csv");
    assert_eq!(got.currency, "EUR");
    assert_eq!(got.status, BatchStatus::Parsed);
    assert!(got.account_id.is_none());
    assert_eq!(got.headers, vec!["Fecha", "Valor", "Desc"]);
    assert_eq!(got.detected_delimiter, ";");

    let list = repo.list().await.expect("list");
    assert_eq!(list.len(), 1);

    repo.update_status(id, BatchStatus::Reconciled)
        .await
        .expect("update_status");
    let updated = repo.get(id).await.expect("get after update");
    assert_eq!(updated.status, BatchStatus::Reconciled);

    common::clean(&pool).await;
}

#[tokio::test]
async fn find_by_filename_and_delete_batch() {
    let pool = common::setup_pool().await;
    common::clean(&pool).await;
    let repo = MysqlBatchRepository::new(pool.clone());

    let id = repo
        .insert(NewBatch {
            filename: "dup-check.csv".into(),
            file_hash: "hash-find".into(),
            format: "csv".into(),
            currency: "EUR".into(),
            account_id: None,
            status: BatchStatus::Parsed,
            headers: vec![],
            detected_delimiter: ",".into(),
        })
        .await
        .expect("insert");

    let found = repo.find_by_filename("dup-check.csv").await.expect("find");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, id);

    repo.delete(id).await.expect("delete");
    let found = repo.find_by_filename("dup-check.csv").await.expect("find");
    assert!(found.is_empty());
    assert!(repo.get(id).await.is_err());

    common::clean(&pool).await;
}
