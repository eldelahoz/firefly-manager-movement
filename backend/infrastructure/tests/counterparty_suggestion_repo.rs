mod common;

use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use ffm_application::dtos::{CounterpartyMemoryEntry, CounterpartyRef, NewBatch, NewMovement};
use ffm_application::ports::{
    BatchRepository, CounterpartySuggestionRepository, MovementRepository,
};
use ffm_domain::ids::{BatchId, MovementId};
use ffm_domain::transaction_kind::Direction;
use ffm_domain::value_objects::{AmountCents, BatchStatus, MovementStatus};
use ffm_infrastructure::persistence::repositories::batch::SqliteBatchRepository;
use ffm_infrastructure::persistence::repositories::counterparty_suggestion::SqliteCounterpartySuggestionRepository;
use ffm_infrastructure::persistence::repositories::movement::SqliteMovementRepository;
use ffm_infrastructure::schema::counterparty_memory;

fn entry(
    account: i64,
    direction: Direction,
    key: &str,
    cp: CounterpartyRef,
) -> CounterpartyMemoryEntry {
    CounterpartyMemoryEntry {
        account_ff3_id: account,
        direction,
        desc_key: key.into(),
        counterparty: cp,
    }
}

fn by_id(id: i64, name: &str) -> CounterpartyRef {
    CounterpartyRef {
        ff3_id: Some(id),
        name: Some(name.into()),
        cash: false,
    }
}

fn cash() -> CounterpartyRef {
    CounterpartyRef {
        cash: true,
        ..Default::default()
    }
}

#[tokio::test]
async fn remember_upserts_latest_choice_and_counts_uses() {
    let pool = common::setup_pool().await;
    let repo = SqliteCounterpartySuggestionRepository::new(pool.clone());

    repo.remember(entry(
        7,
        Direction::Out,
        "pago pse acme motors",
        by_id(10, "Acme Motors"),
    ))
    .await
    .unwrap();
    repo.remember(entry(7, Direction::Out, "pago pse acme motors", cash()))
        .await
        .unwrap();
    // Same key, other direction and other account: separate entries.
    repo.remember(entry(
        7,
        Direction::In,
        "pago pse acme motors",
        by_id(20, "Refund"),
    ))
    .await
    .unwrap();
    repo.remember(entry(
        8,
        Direction::Out,
        "pago pse acme motors",
        by_id(11, "Other"),
    ))
    .await
    .unwrap();

    let mut learned = repo.learned(7).await.unwrap();
    learned.sort_by_key(|e| e.direction.as_str());
    assert_eq!(
        learned,
        vec![
            entry(
                7,
                Direction::In,
                "pago pse acme motors",
                by_id(20, "Refund")
            ),
            entry(7, Direction::Out, "pago pse acme motors", cash()),
        ]
    );

    let mut conn = pool.get().await.unwrap();
    let uses: i64 = counterparty_memory::table
        .filter(counterparty_memory::account_ff3_id.eq(7))
        .filter(counterparty_memory::direction.eq("out"))
        .select(counterparty_memory::uses)
        .first(&mut conn)
        .await
        .unwrap();
    assert_eq!(uses, 2);
}

#[tokio::test]
async fn name_only_choice_round_trips() {
    let pool = common::setup_pool().await;
    let repo = SqliteCounterpartySuggestionRepository::new(pool);
    let named = CounterpartyRef {
        name: Some("Corner Shop".into()),
        ..Default::default()
    };
    repo.remember(entry(7, Direction::Out, "corner shop", named.clone()))
        .await
        .unwrap();
    assert_eq!(repo.learned(7).await.unwrap()[0].counterparty, named);
}

async fn batch_with_movements(pool: &common::DbPool) -> (BatchId, Vec<MovementId>) {
    let batches = SqliteBatchRepository::new(pool.clone());
    let movements = SqliteMovementRepository::new(pool.clone());
    let batch_id = batches
        .insert(NewBatch {
            filename: "s.csv".into(),
            file_hash: "h".into(),
            format: "csv".into(),
            currency: "EUR".into(),
            account_id: None,
            status: BatchStatus::Parsed,
            headers: vec![],
            detected_delimiter: ",".into(),
        })
        .await
        .unwrap();
    movements
        .insert_batch(
            (1..=3)
                .map(|line_no| NewMovement {
                    batch_id,
                    line_no,
                    date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
                    amount_cents: AmountCents::new(-100),
                    description: format!("m{line_no}"),
                    raw_json: "{}".into(),
                    status: MovementStatus::Missing,
                })
                .collect(),
        )
        .await
        .unwrap();
    let ids = movements
        .by_batch(batch_id)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.id)
        .collect();
    (batch_id, ids)
}

#[tokio::test]
async fn save_history_replaces_the_batch_suggestions() {
    let pool = common::setup_pool().await;
    let repo = SqliteCounterpartySuggestionRepository::new(pool.clone());
    let (batch_id, ids) = batch_with_movements(&pool).await;

    repo.save_history(
        batch_id,
        vec![
            (ids[0], by_id(10, "Acme Motors")),
            (ids[1], by_id(11, "Shop")),
        ],
    )
    .await
    .unwrap();
    let first = repo.history_by_batch(batch_id).await.unwrap();
    assert_eq!(first.len(), 2);
    assert_eq!(first[&ids[0]], by_id(10, "Acme Motors"));

    // A second reconcile clears what it no longer suggests.
    repo.save_history(batch_id, vec![(ids[2], by_id(12, "Cafe"))])
        .await
        .unwrap();
    let second = repo.history_by_batch(batch_id).await.unwrap();
    assert_eq!(second.len(), 1);
    assert_eq!(second[&ids[2]], by_id(12, "Cafe"));
}
