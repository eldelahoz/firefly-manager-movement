use chrono::NaiveDate;
use std::str::FromStr;

use ffm_application::error::ApplicationError;
use ffm_application::ports::FireflyClient;
use ffm_domain::value_objects::AmountCents;
use ffm_infrastructure::firefly::client::ReqwestFireflyClient;
use wiremock::{
    matchers::{header, method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

fn make_client(server: &MockServer) -> ReqwestFireflyClient {
    ReqwestFireflyClient::new(&server.uri(), "test-token")
}

#[tokio::test]
async fn about_ok() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/about"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": { "version": "6.1.36" }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let about = client.about().await.unwrap();
    assert_eq!(about.version, "6.1.36");
}

#[tokio::test]
async fn about_unreachable() {
    let client = ReqwestFireflyClient::new("http://127.0.0.1:1", "x");
    let err = client.about().await.unwrap_err();
    match &err {
        ApplicationError::FireflyClient(msg) => {
            assert!(msg.contains("127.0.0.1:1") || msg.to_lowercase().contains("connection"))
        }
        other => panic!("expected FireflyClient error, got {:?}", other),
    }
}

#[tokio::test]
async fn asset_accounts_single_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts"))
        .and(query_param("type", "asset"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "1",
                    "attributes": { "name": "Main Bank", "type": "asset", "currency_code": "EUR" }
                },
                {
                    "id": "2",
                    "attributes": { "name": "Savings", "type": "asset", "currency_code": "USD" }
                }
            ],
            "meta": { "pagination": { "current_page": 1, "total_pages": 1 } }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let accounts = client.asset_accounts().await.unwrap();
    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts[0].ff3_id, 1);
    assert_eq!(accounts[0].name, "Main Bank");
    assert_eq!(accounts[0].account_type, "asset");
    assert_eq!(accounts[0].currency, "EUR");
    assert_eq!(accounts[1].ff3_id, 2);
    assert_eq!(accounts[1].name, "Savings");
    assert_eq!(accounts[1].currency, "USD");
}

#[tokio::test]
async fn asset_accounts_maps_iban_and_account_number() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts"))
        .and(query_param("type", "asset"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "1",
                    "attributes": {
                        "name": "Main Bank",
                        "type": "asset",
                        "currency_code": "EUR",
                        "iban": "DE44 5001 0517 5407 3249 31",
                        "account_number": "1001"
                    }
                },
                {
                    "id": "2",
                    "attributes": { "name": "Savings", "type": "asset", "currency_code": "USD" }
                }
            ],
            "meta": { "pagination": { "current_page": 1, "total_pages": 1 } }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let accounts = client.asset_accounts().await.unwrap();
    assert_eq!(
        accounts[0].iban.as_deref(),
        Some("DE44 5001 0517 5407 3249 31")
    );
    assert_eq!(accounts[0].account_number.as_deref(), Some("1001"));
    // absent attributes map to None (serde defaults)
    assert_eq!(accounts[1].iban, None);
    assert_eq!(accounts[1].account_number, None);
}

#[tokio::test]
async fn accounts_by_type_maps_role_and_liability_type() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts"))
        .and(query_param("type", "liability"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "7",
                    "attributes": {
                        "name": "Prestamo Moto",
                        "type": "liabilities",
                        "currency_code": "COP",
                        "account_role": null,
                        "liability_type": "loan"
                    }
                },
                {
                    "id": "4",
                    "attributes": {
                        "name": "Tarjeta",
                        "type": "asset",
                        "currency_code": "COP",
                        "account_role": "ccAsset"
                    }
                }
            ],
            "meta": { "pagination": { "current_page": 1, "total_pages": 1 } }
        })))
        .mount(&server)
        .await;

    let accounts = make_client(&server)
        .accounts_by_type("liability")
        .await
        .unwrap();
    assert_eq!(accounts[0].liability_type.as_deref(), Some("loan"));
    assert_eq!(accounts[0].role, None);
    assert_eq!(accounts[1].role.as_deref(), Some("ccAsset"));
    assert_eq!(accounts[1].liability_type, None);
}

#[tokio::test]
async fn asset_accounts_paginates() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts"))
        .and(query_param("type", "asset"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "id": "1",
                "attributes": { "name": "A", "type": "asset", "currency_code": "EUR" }
            }],
            "meta": { "pagination": { "current_page": 1, "total_pages": 2 } }
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts"))
        .and(query_param("type", "asset"))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "id": "2",
                "attributes": { "name": "B", "type": "asset", "currency_code": "GBP" }
            }],
            "meta": { "pagination": { "current_page": 2, "total_pages": 2 } }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let accounts = client.asset_accounts().await.unwrap();
    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts[1].name, "B");
    assert_eq!(accounts[1].currency, "GBP");
}

#[tokio::test]
async fn account_transactions_single_page() {
    let server = MockServer::start().await;
    // Real FF3 format: `amount` is positive; the sense is conveyed by `type`.
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "id": "42",
                "attributes": {
                    "transactions": [{
                        "type": "withdrawal",
                        "date": "2026-01-05T00:00:00+00:00",
                        "amount": "123.45",
                        "description": "lunch",
                        "source_id": "1",
                        "destination_id": "7"
                    }]
                }
            }],
            "meta": { "pagination": { "current_page": 1, "total_pages": 1 } }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
    let txs = client
        .account_transactions(1, Some(start), Some(end))
        .await
        .unwrap();

    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0].ff3_id, 42);
    assert_eq!(txs[0].date, NaiveDate::from_ymd_opt(2026, 1, 5).unwrap());
    assert_eq!(txs[0].amount_cents.as_i64(), -12345);
    assert_eq!(txs[0].description, "lunch");
    assert!(txs[0].splits.is_empty());
}

#[tokio::test]
async fn account_transactions_paginates() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/5/transactions"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "10",
                    "attributes": {
                        "transactions": [{
                            "type": "deposit",
                            "date": "2026-02-01T00:00:00+00:00",
                            "amount": "50.00",
                            "description": "salary",
                            "source_id": "9",
                            "destination_id": "5"
                        }]
                    }
                },
                {
                    "id": "11",
                    "attributes": {
                        "transactions": [{
                            "type": "withdrawal",
                            "date": "2026-02-03T00:00:00+00:00",
                            "amount": "25.50",
                            "description": "groceries",
                            "source_id": "5",
                            "destination_id": "8"
                        }]
                    }
                }
            ],
            "meta": { "pagination": { "current_page": 1, "total_pages": 2 } }
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/5/transactions"))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "id": "12",
                "attributes": {
                    "transactions": [{
                        "type": "withdrawal",
                        "date": "2026-02-15T00:00:00+00:00",
                        "amount": "100.00",
                        "description": "rent",
                        "source_id": "5",
                        "destination_id": "9"
                    }]
                }
            }],
            "meta": { "pagination": { "current_page": 2, "total_pages": 2 } }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let start = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 2, 28).unwrap();
    let txs = client
        .account_transactions(5, Some(start), Some(end))
        .await
        .unwrap();

    assert_eq!(txs.len(), 3);
    assert_eq!(txs[0].description, "salary");
    assert_eq!(txs[1].description, "groceries");
    assert_eq!(txs[2].description, "rent");
    assert_eq!(txs[2].amount_cents.as_i64(), -10000);
}

#[tokio::test]
async fn account_transactions_transfer_sign_follows_queried_account() {
    let server = MockServer::start().await;
    // Out of account 5 (source) -> negative; into account 5 (destination) -> positive.
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/5/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "21",
                    "attributes": {
                        "transactions": [{
                            "type": "transfer",
                            "date": "2026-03-01T00:00:00+00:00",
                            "amount": "300.00",
                            "description": "to savings",
                            "source_id": "5",
                            "destination_id": "6"
                        }]
                    }
                },
                {
                    "id": "22",
                    "attributes": {
                        "transactions": [{
                            "type": "transfer",
                            "date": "2026-03-02T00:00:00+00:00",
                            "amount": "75.00",
                            "description": "from savings",
                            "source_id": "6",
                            "destination_id": "5"
                        }]
                    }
                }
            ],
            "meta": { "pagination": { "current_page": 1, "total_pages": 1 } }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let start = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 3, 31).unwrap();
    let txs = client
        .account_transactions(5, Some(start), Some(end))
        .await
        .unwrap();

    assert_eq!(txs.len(), 2);
    assert_eq!(txs[0].ff3_id, 21);
    assert_eq!(txs[0].amount_cents.as_i64(), -30000);
    assert_eq!(txs[1].ff3_id, 22);
    assert_eq!(txs[1].amount_cents.as_i64(), 7500);
}

#[tokio::test]
async fn account_transactions_aggregates_split_group() {
    let server = MockServer::start().await;
    // Real FF3 payload: one group (id 48) split in two journals (49, 48).
    // The statement shows the group total, so the run must see a single
    // transaction of -1000.00, not the first split.
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "id": "48",
                "attributes": {
                    "group_title": "Prueba split FFM",
                    "transactions": [
                        {
                            "transaction_journal_id": "49",
                            "type": "withdrawal",
                            "date": "2026-10-18T07:00:00-05:00",
                            "amount": "600.000000000000",
                            "description": "Split parte A",
                            "source_id": "1",
                            "destination_id": "5"
                        },
                        {
                            "transaction_journal_id": "48",
                            "type": "withdrawal",
                            "date": "2026-10-18T07:00:00-05:00",
                            "amount": "400.000000000000",
                            "description": "Split parte B",
                            "source_id": "1",
                            "destination_id": "5"
                        }
                    ]
                }
            }],
            "meta": { "pagination": { "current_page": 1, "total_pages": 1 } }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let start = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 10, 20).unwrap();
    let txs = client
        .account_transactions(1, Some(start), Some(end))
        .await
        .unwrap();

    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0].ff3_id, 48);
    assert_eq!(txs[0].date, NaiveDate::from_ymd_opt(2026, 10, 18).unwrap());
    assert_eq!(txs[0].amount_cents.as_i64(), -100000);
    assert_eq!(txs[0].description, "Prueba split FFM");
    // The statement may also show each split as its own line instead.
    let splits: Vec<_> = txs[0]
        .splits
        .iter()
        .map(|s| {
            (
                s.journal_id,
                s.amount_cents.as_i64(),
                s.description.as_str(),
            )
        })
        .collect();
    assert_eq!(
        splits,
        vec![(49, -60000, "Split parte A"), (48, -40000, "Split parte B")]
    );
    assert_eq!(txs[0].splits[0].date, txs[0].date);
}

#[tokio::test]
async fn account_transactions_split_without_group_title_joins_descriptions() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "id": "60",
                "attributes": {
                    "group_title": "",
                    "transactions": [
                        {
                            "transaction_journal_id": "61",
                            "type": "withdrawal",
                            "date": "2026-10-01T12:00:00+00:00",
                            "amount": "30.00",
                            "description": "Supermercado",
                            "source_id": "1",
                            "destination_id": "5"
                        },
                        {
                            "transaction_journal_id": "62",
                            "type": "withdrawal",
                            "date": "2026-10-01T12:00:00+00:00",
                            "amount": "20.00",
                            "description": "Farmacia",
                            "source_id": "1",
                            "destination_id": "5"
                        },
                        {
                            "transaction_journal_id": "63",
                            "type": "withdrawal",
                            "date": "2026-10-02T12:00:00+00:00",
                            "amount": "10.00",
                            "description": "Farmacia",
                            "source_id": "1",
                            "destination_id": "5"
                        }
                    ]
                }
            }],
            "meta": { "pagination": { "current_page": 1, "total_pages": 1 } }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let start = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 10, 31).unwrap();
    let txs = client
        .account_transactions(1, Some(start), Some(end))
        .await
        .unwrap();

    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0].amount_cents.as_i64(), -6000);
    // Earliest split date wins, descriptions are deduplicated in order.
    assert_eq!(txs[0].date, NaiveDate::from_ymd_opt(2026, 10, 1).unwrap());
    assert_eq!(txs[0].description, "Supermercado / Farmacia");
}

#[tokio::test]
async fn account_transactions_split_excludes_splits_of_other_accounts() {
    let server = MockServer::start().await;
    // Split group where one journal moves money between two OTHER accounts:
    // it does not touch account 5, so it must not be summed.
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/5/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "id": "70",
                "attributes": {
                    "transactions": [
                        {
                            "transaction_journal_id": "72",
                            "type": "withdrawal",
                            "date": "2026-04-01T00:00:00+00:00",
                            "amount": "999.00",
                            "description": "other accounts",
                            "source_id": "6",
                            "destination_id": "8"
                        },
                        {
                            "transaction_journal_id": "71",
                            "type": "withdrawal",
                            "date": "2026-04-01T00:00:00+00:00",
                            "amount": "45.00",
                            "description": "from account 5",
                            "source_id": "5",
                            "destination_id": "8"
                        }
                    ]
                }
            }],
            "meta": { "pagination": { "current_page": 1, "total_pages": 1 } }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let start = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 4, 30).unwrap();
    let txs = client
        .account_transactions(5, Some(start), Some(end))
        .await
        .unwrap();

    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0].amount_cents.as_i64(), -4500);
    assert_eq!(txs[0].description, "from account 5");
}

#[tokio::test]
async fn account_transactions_split_netting_to_zero_is_skipped() {
    let server = MockServer::start().await;
    // Money out of and back into the same account within one group: no net
    // movement for the statement, so the group must be dropped.
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/5/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "80",
                    "attributes": {
                        "transactions": [
                            {
                                "transaction_journal_id": "81",
                                "type": "withdrawal",
                                "date": "2026-05-01T00:00:00+00:00",
                                "amount": "50.00",
                                "description": "out",
                                "source_id": "5",
                                "destination_id": "9"
                            },
                            {
                                "transaction_journal_id": "82",
                                "type": "deposit",
                                "date": "2026-05-01T00:00:00+00:00",
                                "amount": "50.00",
                                "description": "back in",
                                "source_id": "9",
                                "destination_id": "5"
                            }
                        ]
                    }
                },
                {
                    "id": "83",
                    "attributes": {
                        "transactions": [{
                            "transaction_journal_id": "84",
                            "type": "withdrawal",
                            "date": "2026-05-02T00:00:00+00:00",
                            "amount": "10.00",
                            "description": "kept",
                            "source_id": "5",
                            "destination_id": "9"
                        }]
                    }
                }
            ],
            "meta": { "pagination": { "current_page": 1, "total_pages": 1 } }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let start = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();
    let txs = client
        .account_transactions(5, Some(start), Some(end))
        .await
        .unwrap();

    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0].ff3_id, 83);
}

#[tokio::test]
async fn account_transactions_single_day_window_is_widened_and_clamped() {
    let server = MockServer::start().await;
    // FF3 rejects start == end (422). The client must widen the query by a
    // day and drop the surplus transaction it brings back.
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/transactions"))
        .and(query_param("start", "2026-10-18"))
        .and(query_param("end", "2026-10-19"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "90",
                    "attributes": {
                        "transactions": [{
                            "transaction_journal_id": "90",
                            "type": "withdrawal",
                            "date": "2026-10-18T00:00:00+00:00",
                            "amount": "10.00",
                            "description": "in window",
                            "source_id": "1",
                            "destination_id": "5"
                        }]
                    }
                },
                {
                    "id": "91",
                    "attributes": {
                        "transactions": [{
                            "transaction_journal_id": "91",
                            "type": "withdrawal",
                            "date": "2026-10-19T00:00:00+00:00",
                            "amount": "20.00",
                            "description": "past window",
                            "source_id": "1",
                            "destination_id": "5"
                        }]
                    }
                }
            ],
            "meta": { "pagination": { "current_page": 1, "total_pages": 1 } }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let day = NaiveDate::from_ymd_opt(2026, 10, 18).unwrap();
    let txs = client
        .account_transactions(1, Some(day), Some(day))
        .await
        .unwrap();

    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0].ff3_id, 90);
}

#[tokio::test]
async fn account_transactions_split_only_counts_journals_inside_the_window() {
    let server = MockServer::start().await;
    // One group whose splits straddle the window edge: only the in-window
    // journal belongs to the reconciled period.
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "id": "95",
                "attributes": {
                    "transactions": [
                        {
                            "transaction_journal_id": "95",
                            "type": "withdrawal",
                            "date": "2026-10-31T00:00:00+00:00",
                            "amount": "70.00",
                            "description": "October part",
                            "source_id": "1",
                            "destination_id": "5"
                        },
                        {
                            "transaction_journal_id": "96",
                            "type": "withdrawal",
                            "date": "2026-11-02T00:00:00+00:00",
                            "amount": "30.00",
                            "description": "November part",
                            "source_id": "1",
                            "destination_id": "5"
                        }
                    ]
                }
            }],
            "meta": { "pagination": { "current_page": 1, "total_pages": 1 } }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let start = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 10, 31).unwrap();
    let txs = client
        .account_transactions(1, Some(start), Some(end))
        .await
        .unwrap();

    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0].amount_cents.as_i64(), -7000);
    assert_eq!(txs[0].description, "October part");
}

#[tokio::test]
async fn account_transactions_amount_parse_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "id": "99",
                "attributes": {
                    "transactions": [{
                        "date": "2026-01-05T00:00:00+00:00",
                        "amount": "not-a-number",
                        "description": "bad"
                    }]
                }
            }],
            "meta": { "pagination": { "current_page": 1, "total_pages": 1 } }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
    let err = client
        .account_transactions(1, Some(start), Some(end))
        .await
        .unwrap_err();
    match &err {
        ApplicationError::Parse(msg) => assert!(msg.contains("not-a-number")),
        other => panic!("expected Parse error, got {:?}", other),
    }
}

#[tokio::test]
async fn delete_transaction_ok() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/transactions/77"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = make_client(&server);
    client.delete_transaction(77).await.unwrap();
}

#[tokio::test]
async fn delete_transaction_not_found_maps_to_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/transactions/78"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "message": "Resource not found"
        })))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client.delete_transaction(78).await.unwrap_err();
    match &err {
        ApplicationError::NotFound(msg) => assert!(msg.contains("78")),
        other => panic!("expected NotFound error, got {:?}", other),
    }
}

#[tokio::test]
async fn create_transaction_ok() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/transactions"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": { "id": "77" }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let creation = ffm_application::dtos::NewFf3TransactionCreation {
        transaction_type: "withdrawal".into(),
        amount: AmountCents::from_str("123.45").unwrap(),
        date: NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
        description: "test tx".into(),
        source_ff3_id: Some(1),
        destination_ff3_id: Some(2),
        source_name: None,
        destination_name: None,
        category: None,
        budget: None,
        notes: None,
        tags: vec![],
        external_id: None,
    };
    let id = client.create_transaction(creation).await.unwrap().ff3_id;
    assert_eq!(id, 77);
}

#[tokio::test]
async fn create_transaction_wraps_in_transactions_array() {
    let server = MockServer::start().await;
    let _mock = Mock::given(method("POST"))
        .and(path("/api/v1/transactions"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": { "id": "1" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let creation = ffm_application::dtos::NewFf3TransactionCreation {
        transaction_type: "deposit".into(),
        amount: AmountCents::from_str("50.00").unwrap(),
        date: NaiveDate::from_ymd_opt(2026, 9, 3).unwrap(),
        description: "salary".into(),
        source_ff3_id: Some(10),
        destination_ff3_id: Some(20),
        source_name: None,
        destination_name: None,
        category: Some("Income".into()),
        budget: None,
        notes: Some("monthly".into()),
        tags: vec!["tag1".into()],
        external_id: Some("ffm-0123456789abcdef-3".into()),
    };
    let id = client.create_transaction(creation).await.unwrap().ff3_id;
    assert_eq!(id, 1);

    // Verify the request body has transactions array at TOP LEVEL
    // (FF3 does not use the JSON-API data/attributes wrapper on requests)
    let requests = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert!(
        body.get("transactions").is_some(),
        "Expected top-level transactions array, got: {body}"
    );
    let tx = &body["transactions"][0];
    assert_eq!(tx["type"], "deposit");
    assert_eq!(tx["amount"], "50.00");
    assert_eq!(tx["description"], "salary");
    assert_eq!(tx["source_id"], "10");
    assert_eq!(tx["destination_id"], "20");
    assert_eq!(tx["category_name"], "Income");
    assert_eq!(tx["notes"], "monthly");
    assert_eq!(tx["tags"], serde_json::json!(["tag1"]));
    assert_eq!(tx["external_id"], "ffm-0123456789abcdef-3");
    // Date must be UTC noon: FF3 renders/stores in the user's timezone, so
    // midnight UTC shifts the visible day back for any UTC+x zone. Noon keeps
    // the same calendar day for offsets -12..+11 (midnight would not).
    let date_str = tx["date"].as_str().unwrap();
    assert_eq!(
        date_str, "2026-09-03T12:00:00+00:00",
        "Expected UTC-noon date-time, got: {date_str}"
    );
}

#[tokio::test]
async fn create_transaction_sends_absolute_amount() {
    let server = MockServer::start().await;
    let _mock = Mock::given(method("POST"))
        .and(path("/api/v1/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": { "id": "7" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let creation = ffm_application::dtos::NewFf3TransactionCreation {
        transaction_type: "withdrawal".into(),
        amount: AmountCents::from_str("-42.00").unwrap(),
        date: NaiveDate::from_ymd_opt(2026, 8, 3).unwrap(),
        description: "groceries".into(),
        source_ff3_id: Some(1),
        destination_ff3_id: Some(2),
        source_name: None,
        destination_name: None,
        category: None,
        budget: None,
        notes: None,
        tags: vec![],
        external_id: None,
    };
    client.create_transaction(creation).await.unwrap();

    // FF3 requires a positive amount; the sign is conveyed by
    // type + source/destination, never by the amount itself.
    let requests = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    let tx = &body["transactions"][0];
    assert_eq!(tx["type"], "withdrawal");
    assert_eq!(tx["amount"], "42.00");
}

#[tokio::test]
async fn create_transaction_omits_absent_account_side() {
    let server = MockServer::start().await;
    let _mock = Mock::given(method("POST"))
        .and(path("/api/v1/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": { "id": "9" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let creation = ffm_application::dtos::NewFf3TransactionCreation {
        transaction_type: "withdrawal".into(),
        amount: AmountCents::from_str("-1.00").unwrap(),
        date: NaiveDate::from_ymd_opt(2026, 8, 3).unwrap(),
        description: "no counterparty".into(),
        source_ff3_id: Some(1),
        destination_ff3_id: None,
        source_name: None,
        destination_name: None,
        category: None,
        budget: None,
        notes: None,
        tags: vec![],
        external_id: None,
    };
    client.create_transaction(creation).await.unwrap();

    let requests = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    let tx = &body["transactions"][0];
    assert_eq!(tx["source_id"], "1");
    assert!(
        tx.get("destination_id").is_none(),
        "counterparty side must be omitted, got: {tx}"
    );
}

#[tokio::test]
async fn create_transaction_sends_transfer_and_counterparty_name() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": { "id": "77" }
        })))
        .expect(2)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let base = ffm_application::dtos::NewFf3TransactionCreation {
        transaction_type: "transfer".into(),
        amount: AmountCents::from_str("-25.00").unwrap(),
        date: NaiveDate::from_ymd_opt(2026, 9, 3).unwrap(),
        description: "pay card".into(),
        source_ff3_id: Some(1),
        destination_ff3_id: Some(4),
        source_name: None,
        destination_name: None,
        category: None,
        budget: None,
        notes: None,
        tags: vec![],
        external_id: None,
    };
    client.create_transaction(base.clone()).await.unwrap();
    client
        .create_transaction(ffm_application::dtos::NewFf3TransactionCreation {
            transaction_type: "deposit".into(),
            amount: AmountCents::from_str("10.00").unwrap(),
            source_ff3_id: None,
            destination_ff3_id: Some(1),
            source_name: Some("New Employer".into()),
            ..base
        })
        .await
        .unwrap();

    let requests = server.received_requests().await.unwrap();
    let tx = |i: usize| -> serde_json::Value {
        let body: serde_json::Value = serde_json::from_slice(&requests[i].body).unwrap();
        body["transactions"][0].clone()
    };
    let transfer = tx(0);
    assert_eq!(transfer["type"], "transfer");
    assert_eq!(transfer["amount"], "25.00");
    assert_eq!(transfer["source_id"], "1");
    assert_eq!(transfer["destination_id"], "4");
    assert!(transfer.get("source_name").is_none());
    let deposit = tx(1);
    assert_eq!(deposit["type"], "deposit");
    assert_eq!(deposit["source_name"], "New Employer");
    assert!(deposit.get("source_id").is_none());
    assert_eq!(deposit["destination_id"], "1");
}

#[tokio::test]
async fn account_transactions_counterparty_is_the_opposite_side() {
    let server = MockServer::start().await;
    let split =
        |src: &str, src_name: &str, src_type: &str, dst: &str, dst_name: &str, dst_type: &str| {
            serde_json::json!({
                "date": "2026-01-05T00:00:00+00:00",
                "amount": "10.00",
                "description": "x",
                "type": "withdrawal",
                "source_id": src, "source_name": src_name, "source_type": src_type,
                "destination_id": dst, "destination_name": dst_name, "destination_type": dst_type
            })
        };
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                { "id": "1", "attributes": { "transactions": [
                    split("1", "Bank", "Asset account", "5", "Planautos S A", "Expense account")
                ] } },
                { "id": "2", "attributes": { "transactions": [
                    split("7", "Prestamo Moto", "Loan", "1", "Bank", "Asset account")
                ] } },
                { "id": "3", "attributes": { "transactions": [{
                    "date": "2026-01-05T00:00:00+00:00", "amount": "1.00",
                    "description": "old", "source_id": "1", "destination_id": "9"
                }] } }
            ],
            "meta": { "pagination": { "current_page": 1, "total_pages": 1 } }
        })))
        .mount(&server)
        .await;

    let txs = make_client(&server)
        .account_transactions(
            1,
            Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(2026, 1, 31).unwrap()),
        )
        .await
        .unwrap();
    let by_id = |id: i64| txs.iter().find(|t| t.ff3_id == id).unwrap();
    assert_eq!(by_id(1).counterparty_name.as_deref(), Some("Planautos S A"));
    assert_eq!(
        by_id(1).counterparty_type.as_deref(),
        Some("Expense account")
    );
    assert_eq!(by_id(2).counterparty_name.as_deref(), Some("Prestamo Moto"));
    assert_eq!(by_id(2).counterparty_type.as_deref(), Some("Loan"));
    assert_eq!(by_id(3).counterparty_name, None);
    assert_eq!(by_id(3).counterparty_type, None);
}

#[tokio::test]
async fn create_transaction_returns_both_account_sides() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "88",
                "attributes": { "transactions": [{
                    "source_id": "1", "source_name": "Bancolombia Ahorros", "source_type": "Asset account",
                    "destination_id": "7", "destination_name": "Prestamo Moto", "destination_type": "Loan"
                }] }
            }
        })))
        .mount(&server)
        .await;

    let created = make_client(&server)
        .create_transaction(ffm_application::dtos::NewFf3TransactionCreation {
            transaction_type: "withdrawal".into(),
            amount: AmountCents::from_str("-5.00").unwrap(),
            date: NaiveDate::from_ymd_opt(2026, 9, 3).unwrap(),
            description: "loan payment".into(),
            source_ff3_id: Some(1),
            destination_ff3_id: Some(7),
            source_name: None,
            destination_name: None,
            category: None,
            budget: None,
            notes: None,
            tags: vec![],
            external_id: None,
        })
        .await
        .unwrap();
    assert_eq!(created.ff3_id, 88);
    assert_eq!(created.source_name.as_deref(), Some("Bancolombia Ahorros"));
    assert_eq!(created.source_type.as_deref(), Some("Asset account"));
    assert_eq!(created.destination_name.as_deref(), Some("Prestamo Moto"));
    assert_eq!(created.destination_type.as_deref(), Some("Loan"));
}

#[tokio::test]
async fn search_by_external_id_prefix_paginates_with_encoded_query() {
    let server = MockServer::start().await;
    let query = "account_id:7 external_id_starts:\"ffm-0123456789abcdef-\"";
    let split = |ext: Option<&str>, src: &str, dst: &str| {
        serde_json::json!({
            "external_id": ext,
            "source_id": src, "source_name": format!("acc {src}"), "source_type": format!("type {src}"),
            "destination_id": dst, "destination_name": format!("acc {dst}"), "destination_type": format!("type {dst}")
        })
    };
    let group = |id: &str, split: serde_json::Value| serde_json::json!({ "id": id, "attributes": { "transactions": [split] } });
    let page = |n: u32, data: serde_json::Value| {
        serde_json::json!({
            "data": data,
            "meta": { "pagination": { "current_page": n, "total_pages": 2 } }
        })
    };
    Mock::given(method("GET"))
        .and(path("/api/v1/search/transactions"))
        .and(query_param("query", query))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            1,
            serde_json::json!([
                group("11", split(Some("ffm-0123456789abcdef-1"), "7", "20")),
                group("12", split(None, "7", "20")),
                // Firefly III's prefix match is not trusted: other ids are dropped.
                group("14", split(Some("ffm-otro-1"), "7", "20")),
                // Same file on another account is not this batch's line.
                group("15", split(Some("ffm-0123456789abcdef-5"), "99", "20")),
            ]),
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/search/transactions"))
        .and(query_param("query", query))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            2,
            serde_json::json!([group(
                "13",
                split(Some("ffm-0123456789abcdef-12"), "30", "7")
            )]),
        )))
        .expect(1)
        .mount(&server)
        .await;

    let hits = make_client(&server)
        .transactions_by_external_id_prefix(7, "ffm-0123456789abcdef-")
        .await
        .unwrap();
    let hit = |ext: &str, ff3_id: i64, cp: &str| ffm_application::dtos::ExternalIdHit {
        external_id: ext.into(),
        ff3_id,
        counterparty_name: Some(format!("acc {cp}")),
        counterparty_type: Some(format!("type {cp}")),
    };
    assert_eq!(
        hits,
        vec![
            hit("ffm-0123456789abcdef-1", 11, "20"),
            hit("ffm-0123456789abcdef-12", 13, "30"),
        ]
    );
    let raw = server.received_requests().await.unwrap()[0]
        .url
        .query()
        .unwrap()
        .to_string();
    assert!(
        raw.contains("query=account_id%3A7+external_id_starts%3A%22ffm-0123456789abcdef-%22"),
        "query must be URL-encoded: {raw}"
    );
}

#[tokio::test]
async fn create_transaction_timeout_warns_it_may_exist() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/transactions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_secs(35))
                .set_body_json(serde_json::json!({ "data": { "id": "1" } })),
        )
        .mount(&server)
        .await;
    let err = make_client(&server)
        .create_transaction(ffm_application::dtos::NewFf3TransactionCreation {
            transaction_type: "withdrawal".into(),
            amount: AmountCents::new(-100),
            date: NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
            description: "slow".into(),
            source_ff3_id: Some(1),
            destination_ff3_id: None,
            source_name: None,
            destination_name: None,
            category: None,
            budget: None,
            notes: None,
            tags: vec![],
            external_id: Some("ffm-x-1".into()),
        })
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("it may have been created in Firefly III; wait a moment before retrying"),
        "{err}"
    );
}

#[tokio::test]
async fn search_by_external_id_prefix_propagates_errors() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/search/transactions"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let err = make_client(&server)
        .transactions_by_external_id_prefix(7, "ffm-x-")
        .await
        .unwrap_err();
    assert!(matches!(err, ApplicationError::FireflyClient(_)), "{err:?}");
}
