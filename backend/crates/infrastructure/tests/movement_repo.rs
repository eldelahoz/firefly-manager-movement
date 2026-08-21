mod common;

use chrono::NaiveDate;
use ffm_application::dtos::{NewBatch, NewMovement};
use ffm_application::ports::{BatchRepository, MovementRepository};
use ffm_domain::value_objects::{AmountCents, BatchStatus, MovementStatus};
use ffm_infrastructure::persistence::repositories::batch::MysqlBatchRepository;
use ffm_infrastructure::persistence::repositories::movement::MysqlMovementRepository;

#[tokio::test]
async fn insert_batch_and_by_batch_movements() {
    let pool = common::setup_pool().await;
    common::clean(&pool).await;
    let batches = MysqlBatchRepository::new(pool.clone());
    let movements = MysqlMovementRepository::new(pool.clone());

    let batch_id = batches
        .insert(NewBatch {
            filename: "m.csv".into(),
            file_hash: "h".into(),
            format: "csv".into(),
            currency: "EUR".into(),
            account_id: None,
            status: BatchStatus::Parsed,
        })
        .await
        .expect("insert batch");

    let to_insert = vec![
        NewMovement {
            batch_id,
            line_no: 1,
            date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
            amount_cents: AmountCents::new(12340),
            description: "lunch".into(),
            raw_json: r#"{"a":1}"#.to_string(),
            status: MovementStatus::Imported,
        },
        NewMovement {
            batch_id,
            line_no: 2,
            date: NaiveDate::from_ymd_opt(2026, 1, 6).unwrap(),
            amount_cents: AmountCents::new(-5000),
            description: "cash".into(),
            raw_json: r#"{"a":2}"#.to_string(),
            status: MovementStatus::Imported,
        },
    ];
    movements
        .insert_batch(to_insert)
        .await
        .expect("insert_batch");

    let got = movements.by_batch(batch_id).await.expect("by_batch");
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].line_no, 1);
    assert_eq!(got[0].amount_cents.as_i64(), 12340);
    assert_eq!(got[1].amount_cents.as_i64(), -5000);
    assert!(got[0].amount_cents.is_credit());
    assert!(got[1].amount_cents.is_debit());

    movements
        .update_status(got[0].id, MovementStatus::Matched)
        .await
        .expect("update_status");
    let after = movements.by_batch(batch_id).await.expect("by_batch after");
    assert_eq!(
        after.iter().find(|m| m.id == got[0].id).unwrap().status,
        MovementStatus::Matched
    );

    common::clean(&pool).await;
}
