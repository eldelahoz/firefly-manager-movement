mod common;

use chrono::NaiveDate;
use ffm_application::dtos::{
    NewBatch, NewCreation, NewFf3Transaction, NewMatch, NewMovement, NewReconciliation,
};
use ffm_application::ports::{
    BatchRepository, CreationRepository, Ff3TransactionRepository, MatchRepository,
    MovementRepository, ReconciliationRepository,
};
use ffm_domain::ids::BatchId;
use ffm_domain::value_objects::{AmountCents, BatchStatus, MovementStatus};
use ffm_infrastructure::persistence::repositories::batch::MysqlBatchRepository;
use ffm_infrastructure::persistence::repositories::creation::MysqlCreationRepository;
use ffm_infrastructure::persistence::repositories::ff3_transaction::MysqlFf3TransactionRepository;
use ffm_infrastructure::persistence::repositories::match_repo::MysqlMatchRepository;
use ffm_infrastructure::persistence::repositories::movement::MysqlMovementRepository;
use ffm_infrastructure::persistence::repositories::reconciliation::MysqlReconciliationRepository;

#[tokio::test]
async fn reconcile_flow_ff3_transactions_matches_creations() {
    let pool = common::setup_pool().await;
    common::clean(&pool).await;
    let batches = MysqlBatchRepository::new(pool.clone());
    let movements = MysqlMovementRepository::new(pool.clone());
    let reconciliations = MysqlReconciliationRepository::new(pool.clone());
    let transactions = MysqlFf3TransactionRepository::new(pool.clone());
    let matches = MysqlMatchRepository::new(pool.clone());
    let creations = MysqlCreationRepository::new(pool.clone());

    let batch_id: BatchId = batches
        .insert(NewBatch {
            filename: "flow.csv".into(),
            file_hash: "h".into(),
            format: "csv".into(),
            currency: "EUR".into(),
            account_id: None,
            status: BatchStatus::Parsed,
        })
        .await
        .expect("insert batch");

    movements
        .insert_batch(vec![
            NewMovement {
                batch_id,
                line_no: 1,
                date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
                amount_cents: AmountCents::new(12340),
                description: "lunch".into(),
                raw_json: r#"{"a":1}"#.into(),
                status: MovementStatus::Imported,
            },
            NewMovement {
                batch_id,
                line_no: 2,
                date: NaiveDate::from_ymd_opt(2026, 1, 6).unwrap(),
                amount_cents: AmountCents::new(-5000),
                description: "cash".into(),
                raw_json: r#"{"a":2}"#.into(),
                status: MovementStatus::Imported,
            },
        ])
        .await
        .expect("insert movements");

    let rec_id = reconciliations
        .insert(NewReconciliation {
            batch_id,
            tolerance_days: 2,
            window_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            window_end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            matched_count: 1,
            missing_count: 1,
            extra_count: 0,
        })
        .await
        .expect("insert reconciliation");

    // ff3_transactions: bulk insert + by_reconciliation ordenado por fecha.
    transactions
        .insert_batch(vec![
            NewFf3Transaction {
                reconciliation_id: rec_id,
                ff3_id: 101,
                date: NaiveDate::from_ymd_opt(2026, 1, 6).unwrap(),
                amount_cents: AmountCents::new(-5000),
                description: "cash".into(),
                is_duplicate: false,
            },
            NewFf3Transaction {
                reconciliation_id: rec_id,
                ff3_id: 102,
                date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
                amount_cents: AmountCents::new(12340),
                description: "lunch".into(),
                is_duplicate: false,
            },
        ])
        .await
        .expect("insert ff3 tx");

    let fetched_txs = transactions
        .by_reconciliation(rec_id)
        .await
        .expect("by_reconciliation");
    assert_eq!(fetched_txs.len(), 2);
    assert_eq!(fetched_txs[0].ff3_id, 102, "orden por fecha asc");
    assert_eq!(fetched_txs[1].ff3_id, 101);

    // matches: apareo 1-a-1 entre un movement y una ff3_transaction.
    let mov = movements.by_batch(batch_id).await.expect("movements");
    let lunch_mov = mov
        .iter()
        .find(|m| m.amount_cents.as_i64() == 12340)
        .unwrap();
    let lunch_tx = fetched_txs.iter().find(|t| t.ff3_id == 102).unwrap();

    matches
        .insert_batch(vec![NewMatch {
            reconciliation_id: rec_id,
            movement_id: lunch_mov.id,
            ff3_transaction_id: lunch_tx.id,
        }])
        .await
        .expect("insert match");

    let fetched_matches = matches.by_reconciliation(rec_id).await.expect("matches");
    assert_eq!(fetched_matches.len(), 1);
    assert_eq!(fetched_matches[0].movement_id, lunch_mov.id);
    assert_eq!(fetched_matches[0].ff3_transaction_id, lunch_tx.id);

    // creations: insert + by_movement + by_batch (vía join con movements).
    let creation_id = creations
        .insert(NewCreation {
            movement_id: lunch_mov.id,
            ff3_id: 500,
        })
        .await
        .expect("insert creation");

    let by_mov = creations
        .by_movement(lunch_mov.id)
        .await
        .expect("by_movement")
        .expect("some");
    assert_eq!(by_mov.id, creation_id);
    assert_eq!(by_mov.ff3_id, 500);

    let by_batch = creations.by_batch(batch_id).await.expect("by_batch");
    assert_eq!(by_batch.len(), 1);
    assert_eq!(by_batch[0].movement_id, lunch_mov.id);
    assert_eq!(by_batch[0].ff3_id, 500);

    let cash_mov = mov
        .iter()
        .find(|m| m.amount_cents.as_i64() == -5000)
        .unwrap();
    assert!(creations
        .by_movement(cash_mov.id)
        .await
        .expect("by_movement none")
        .is_none());

    common::clean(&pool).await;
}
