mod common;

use std::sync::{Mutex, OnceLock};

use ffm_application::dtos::NewFf3Account;
use ffm_application::ports::Ff3AccountRepository;
use ffm_infrastructure::persistence::repositories::ff3_account::MysqlFf3AccountRepository;

/// Serializes tests sharing the dev database (clean() truncates all tables).
fn db_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

fn account(ff3_id: i64, name: &str, account_type: &str) -> NewFf3Account {
    NewFf3Account {
        ff3_id,
        name: name.into(),
        account_type: account_type.into(),
        currency: "EUR".into(),
        iban: None,
        account_number: None,
    }
}

#[allow(clippy::await_holding_lock)]
#[tokio::test]
async fn upsert_stores_iban_and_account_number() {
    let _guard = db_lock();
    let pool = common::setup_pool().await;
    common::clean(&pool).await;
    let repo = MysqlFf3AccountRepository::new(pool.clone());

    repo.upsert_batch(vec![NewFf3Account {
        ff3_id: 7,
        name: "Checking".into(),
        account_type: "asset".into(),
        currency: "EUR".into(),
        iban: Some("DE44 5001 0517 5407 3249 31".into()),
        account_number: Some("1001".into()),
    }])
    .await
    .expect("upsert");

    let a = repo.by_ff3_id(7).await.expect("get").expect("some");
    assert_eq!(a.iban.as_deref(), Some("DE44 5001 0517 5407 3249 31"));
    assert_eq!(a.account_number.as_deref(), Some("1001"));

    // Re-upsert updates iban/number and can clear them (None).
    repo.upsert_batch(vec![NewFf3Account {
        ff3_id: 7,
        name: "Checking".into(),
        account_type: "asset".into(),
        currency: "EUR".into(),
        iban: Some("FR14 2004 1010 0505 0001 3M02 606".into()),
        account_number: None,
    }])
    .await
    .expect("re-upsert");
    let a = repo.by_ff3_id(7).await.expect("get").expect("some");
    assert_eq!(a.iban.as_deref(), Some("FR14 2004 1010 0505 0001 3M02 606"));
    assert_eq!(a.account_number, None);

    common::clean(&pool).await;
}

#[allow(clippy::await_holding_lock)]
#[tokio::test]
async fn upsert_by_ff3_id_and_all_asset() {
    let _guard = db_lock();
    let pool = common::setup_pool().await;
    common::clean(&pool).await;
    let repo = MysqlFf3AccountRepository::new(pool.clone());

    assert!(repo.by_ff3_id(1).await.expect("missing none").is_none());

    repo.upsert_batch(vec![
        account(1, "Cuenta Corriente", "asset"),
        account(2, "Visa", "asset"),
        account(3, "Empleador", "revenue"),
    ])
    .await
    .expect("upsert");

    let a1 = repo.by_ff3_id(1).await.expect("a1").expect("some");
    assert_eq!(a1.ff3_id, 1);
    assert_eq!(a1.name, "Cuenta Corriente");
    assert_eq!(a1.account_type, "asset");

    let assets = repo.all_asset().await.expect("all_asset");
    assert_eq!(assets.len(), 2);
    assert!(assets.iter().all(|a| a.account_type == "asset"));

    // Re-upsert of the same ff3_id with a different name: updates, does not insert.
    repo.upsert_batch(vec![account(1, "Cuenta Corriente Renovada", "asset")])
        .await
        .expect("re-upsert");
    let renamed = repo.by_ff3_id(1).await.expect("renamed").expect("some");
    assert_eq!(renamed.name, "Cuenta Corriente Renovada");
    assert_eq!(repo.all_asset().await.expect("assets after").len(), 2);

    // by_id: local id lookup (used by ApplyMissing to resolve the batch account).
    let by_local = repo.by_id(renamed.id).await.expect("by id").expect("some");
    assert_eq!(by_local.id, renamed.id);
    assert_eq!(by_local.ff3_id, 1);
    assert!(repo
        .by_id(ffm_domain::ids::Ff3AccountId::new(9999))
        .await
        .expect("by id none")
        .is_none());

    common::clean(&pool).await;
}
