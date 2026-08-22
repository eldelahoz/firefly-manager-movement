//! `ReqwestFireflyClient` — Firefly III API v1 adapter.

use std::str::FromStr;

use async_trait::async_trait;
use chrono::NaiveDate;
use serde::Deserialize;

use ffm_application::dtos::{About, Ff3AccountDto, Ff3TransactionDto, NewFf3TransactionCreation};
use ffm_application::error::ApplicationError;
use ffm_application::ports::FireflyClient;
use ffm_domain::value_objects::AmountCents;

// ── Internal FF3 response shapes (infra-only) ──────────────────────────

#[derive(Deserialize)]
struct AboutResponse {
    data: AboutData,
}

#[derive(Deserialize)]
struct AboutData {
    version: String,
}

#[derive(Deserialize)]
struct AccountsResponse {
    data: Vec<AccountData>,
    meta: PaginationMeta,
}

#[derive(Deserialize)]
struct AccountData {
    id: String,
    attributes: AccountAttributes,
}

#[derive(Deserialize)]
struct AccountAttributes {
    name: String,
    #[serde(rename = "type")]
    account_type: String,
    #[serde(rename = "currency_code")]
    currency: String,
}

#[derive(Deserialize)]
struct TransactionsResponse {
    data: Vec<TransactionGroupData>,
    meta: PaginationMeta,
}

#[derive(Deserialize)]
struct TransactionGroupData {
    id: String,
    attributes: TransactionGroupAttributes,
}

#[derive(Deserialize)]
struct TransactionGroupAttributes {
    transactions: Vec<TransactionSplit>,
}

#[derive(Deserialize)]
struct TransactionSplit {
    date: String,
    amount: String,
    description: String,
}

#[derive(Deserialize)]
struct PaginationMeta {
    pagination: PaginationInfo,
}

#[derive(Deserialize)]
struct PaginationInfo {
    #[allow(dead_code)]
    current_page: u32,
    total_pages: u32,
}

#[derive(Deserialize)]
struct CreationResponse {
    data: CreationData,
}

#[derive(Deserialize)]
struct CreationData {
    id: String,
}

// ── Client ─────────────────────────────────────────────────────────────

pub struct ReqwestFireflyClient {
    http: reqwest::Client,
    base_url: String,
    token: String,
}

impl ReqwestFireflyClient {
    pub fn new(base_url: &str, token: &str) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("reqwest client must build"),
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
        }
    }

    async fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<T, ApplicationError> {
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| ApplicationError::FireflyClient(e.to_string()))?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_else(|_| "<no body>".into());
            return Err(ApplicationError::FireflyClient(format!(
                "FF3 returned {status}: {text}"
            )));
        }
        resp.json::<T>()
            .await
            .map_err(|e| ApplicationError::FireflyClient(e.to_string()))
    }

    fn format_decimal(amount: AmountCents) -> String {
        let cents = amount.as_i64();
        let sign = if cents < 0 { "-" } else { "" };
        let abs = cents.unsigned_abs();
        format!("{}{}.{:02}", sign, abs / 100, abs % 100)
    }
}

#[async_trait]
impl FireflyClient for ReqwestFireflyClient {
    async fn about(&self) -> Result<About, ApplicationError> {
        let resp: AboutResponse = self.get_json("/api/v1/about").await?;
        Ok(About {
            version: resp.data.version,
        })
    }

    async fn asset_accounts(&self) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        let mut all = Vec::new();
        let mut page: u32 = 1;
        loop {
            let resp: AccountsResponse = self
                .get_json(&format!("/api/v1/accounts?type=asset&page={page}"))
                .await?;
            let total_pages = resp.meta.pagination.total_pages;
            for acc in resp.data {
                all.push(Ff3AccountDto {
                    ff3_id: acc.id.parse::<i64>().map_err(|e| {
                        ApplicationError::Parse(format!("invalid FF3 account id: {e}"))
                    })?,
                    name: acc.attributes.name,
                    account_type: acc.attributes.account_type,
                    currency: acc.attributes.currency,
                });
            }
            if page >= total_pages {
                break;
            }
            page += 1;
        }
        Ok(all)
    }

    async fn account_transactions(
        &self,
        account_ff3_id: i64,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Ff3TransactionDto>, ApplicationError> {
        let mut all = Vec::new();
        let mut page: u32 = 1;
        let start_str = start.format("%Y-%m-%d");
        let end_str = end.format("%Y-%m-%d");
        loop {
            let resp: TransactionsResponse = self
                .get_json(&format!(
                    "/api/v1/accounts/{account_ff3_id}/transactions?start={start_str}&end={end_str}&page={page}"
                ))
                .await?;
            let total_pages = resp.meta.pagination.total_pages;
            for group in resp.data {
                if let Some(split) = group.attributes.transactions.first() {
                    let date = split
                        .date
                        .get(..10)
                        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                        .ok_or_else(|| {
                            ApplicationError::Parse(format!(
                                "invalid transaction date: {}",
                                split.date
                            ))
                        })?;
                    let amount = AmountCents::from_str(&split.amount).map_err(|e| {
                        ApplicationError::Parse(format!(
                            "invalid transaction amount '{}': {:?}",
                            split.amount, e
                        ))
                    })?;
                    let ff3_id = group.id.parse::<i64>().map_err(|e| {
                        ApplicationError::Parse(format!("invalid FF3 transaction id: {e}"))
                    })?;
                    all.push(Ff3TransactionDto {
                        ff3_id,
                        date,
                        amount_cents: amount,
                        description: split.description.clone(),
                    });
                }
            }
            if page >= total_pages {
                break;
            }
            page += 1;
        }
        Ok(all)
    }

    async fn create_transaction(
        &self,
        creation: NewFf3TransactionCreation,
    ) -> Result<i64, ApplicationError> {
        use serde_json::json;

        let mut attrs = json!({
            "type": creation.transaction_type,
            "amount": Self::format_decimal(creation.amount),
            "date": creation.date.format("%Y-%m-%d").to_string(),
            "description": creation.description,
            "source_id": creation.source_ff3_id.to_string(),
            "destination_id": creation.destination_ff3_id.to_string(),
        });
        let obj = attrs.as_object_mut().unwrap();
        if let Some(ref cat) = creation.category {
            obj.insert("category_name".into(), json!(cat));
        }
        if let Some(ref budget) = creation.budget {
            obj.insert("budget_name".into(), json!(budget));
        }
        if let Some(ref notes) = creation.notes {
            obj.insert("notes".into(), json!(notes));
        }
        if !creation.tags.is_empty() {
            obj.insert("tags".into(), json!(creation.tags));
        }

        let url = format!("{}/api/v1/transactions", self.base_url);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&json!({ "data": { "attributes": attrs } }))
            .send()
            .await
            .map_err(|e| ApplicationError::FireflyClient(e.to_string()))?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_else(|_| "<no body>".into());
            return Err(ApplicationError::FireflyClient(format!(
                "FF3 returned {status}: {text}"
            )));
        }
        let resp: CreationResponse = resp
            .json()
            .await
            .map_err(|e| ApplicationError::FireflyClient(e.to_string()))?;
        resp.data
            .id
            .parse::<i64>()
            .map_err(|e| ApplicationError::Parse(format!("invalid FF3 creation id: {e}")))
    }

    async fn delete_transaction(&self, ff3_id: i64) -> Result<(), ApplicationError> {
        let url = format!("{}/api/v1/transactions/{ff3_id}", self.base_url);
        let resp = self
            .http
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .map_err(|e| ApplicationError::FireflyClient(e.to_string()))?;
        let status = resp.status();
        if status.is_success() || status == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            let text = resp.text().await.unwrap_or_else(|_| "<no body>".into());
            Err(ApplicationError::FireflyClient(format!(
                "FF3 returned {status}: {text}"
            )))
        }
    }
}
