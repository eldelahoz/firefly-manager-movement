mod common;

use ffm_application::dtos::NewFf3Account;
use ffm_application::ports::Ff3AccountRepository;
use ffm_infrastructure::persistence::repositories::ff3_account::MysqlFf3AccountRepository;

fn account(ff3_id: i64, name: &str, account_type: &str) -> NewFf3Account {
    NewFf3Account {
        ff3_id,
        name: name.into(),
        account_type: account_type.into(),
        currency: "EUR".into(),
    }
}

#[tokio::test]
async fn upsert_by_ff3_id_and_all_asset() {
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

    common::clean(&pool).await;
}
