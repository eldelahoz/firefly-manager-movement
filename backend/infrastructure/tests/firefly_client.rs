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
    let txs = client.account_transactions(1, start, end).await.unwrap();

    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0].ff3_id, 42);
    assert_eq!(txs[0].date, NaiveDate::from_ymd_opt(2026, 1, 5).unwrap());
    assert_eq!(txs[0].amount_cents.as_i64(), -12345);
    assert_eq!(txs[0].description, "lunch");
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
    let txs = client.account_transactions(5, start, end).await.unwrap();

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
    let txs = client.account_transactions(5, start, end).await.unwrap();

    assert_eq!(txs.len(), 2);
    assert_eq!(txs[0].ff3_id, 21);
    assert_eq!(txs[0].amount_cents.as_i64(), -30000);
    assert_eq!(txs[1].ff3_id, 22);
    assert_eq!(txs[1].amount_cents.as_i64(), 7500);
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
        .account_transactions(1, start, end)
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
        category: None,
        budget: None,
        notes: None,
        tags: vec![],
    };
    let id = client.create_transaction(creation).await.unwrap();
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
        category: Some("Income".into()),
        budget: None,
        notes: Some("monthly".into()),
        tags: vec!["tag1".into()],
    };
    let id = client.create_transaction(creation).await.unwrap();
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
    // Verify date-time format per FF3 spec
    let date_str = tx["date"].as_str().unwrap();
    assert!(
        date_str.contains('T'),
        "Expected date-time format with 'T', got: {date_str}"
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
        category: None,
        budget: None,
        notes: None,
        tags: vec![],
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
        category: None,
        budget: None,
        notes: None,
        tags: vec![],
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
