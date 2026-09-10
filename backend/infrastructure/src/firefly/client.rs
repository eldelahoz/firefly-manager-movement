//! `ReqwestFireflyClient` - Firefly III API v1 adapter.

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
    #[serde(default)]
    iban: Option<String>,
    #[serde(default)]
    account_number: Option<String>,
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
    #[serde(rename = "type", default)]
    transaction_type: Option<String>,
    #[serde(default)]
    source_id: Option<String>,
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

#[derive(Deserialize)]
struct NamesResponse {
    data: Vec<NamedData>,
    meta: PaginationMeta,
}

#[derive(Deserialize)]
struct NamedData {
    attributes: NamedAttributes,
}

#[derive(Deserialize)]
struct NamedAttributes {
    #[serde(default)]
    tag: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Deserialize)]
struct AttachmentCreateResponse {
    data: AttachmentCreateData,
}

#[derive(Deserialize)]
struct AttachmentCreateData {
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

    async fn paged_accounts(&self, query: &str) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        let mut all = Vec::new();
        let mut page: u32 = 1;
        loop {
            let resp: AccountsResponse = self
                .get_json(&format!("/api/v1/accounts?{query}&page={page}"))
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
                    iban: acc.attributes.iban,
                    account_number: acc.attributes.account_number,
                });
            }
            if page >= total_pages {
                break;
            }
            page += 1;
        }
        Ok(all)
    }

    async fn paged_names(
        &self,
        path: &str,
        extract: impl Fn(&NamedAttributes) -> Option<String> + Send,
    ) -> Result<Vec<String>, ApplicationError> {
        let mut all = Vec::new();
        let mut page: u32 = 1;
        loop {
            let resp: NamesResponse = self.get_json(&format!("{path}?page={page}")).await?;
            let total_pages = resp.meta.pagination.total_pages;
            for item in resp.data {
                if let Some(name) = extract(&item.attributes) {
                    if !name.trim().is_empty() {
                        all.push(name);
                    }
                }
            }
            if page >= total_pages {
                break;
            }
            page += 1;
        }
        all.sort();
        all.dedup();
        Ok(all)
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
        self.paged_accounts("type=asset").await
    }

    async fn accounts_by_type(
        &self,
        account_type: &str,
    ) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        let valid = matches!(account_type, "asset" | "expense" | "revenue" | "liability");
        if !valid {
            return Err(ApplicationError::Parse(format!(
                "invalid account_type: {account_type}"
            )));
        }
        self.paged_accounts(&format!("type={account_type}")).await
    }

    async fn account_transactions(
        &self,
        account_ff3_id: i64,
        start: Option<NaiveDate>,
        end: Option<NaiveDate>,
    ) -> Result<Vec<Ff3TransactionDto>, ApplicationError> {
        let mut all = Vec::new();
        let mut page: u32 = 1;
        // FF3 requires both start and end when one is present (422 otherwise).
        // For open ranges, supply a sensible default: start=1970-01-01, end=today.
        let start_val = start.unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        let end_val = end.unwrap_or_else(|| chrono::Utc::now().date_naive());
        let start_str = start_val.format("%Y-%m-%d");
        let end_str = end_val.format("%Y-%m-%d");
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
                    let magnitude = AmountCents::from_api_str(&split.amount).map_err(|e| {
                        ApplicationError::Parse(format!(
                            "invalid transaction amount '{}': {:?}",
                            split.amount, e
                        ))
                    })?;
                    // FF3 v1 returns `amount` positive and conveys the
                    // sense via `type` (+ source/destination for the rest).
                    // Normalize to the queried account's perspective:
                    // money leaving the account is negative.
                    let account_id_str = account_ff3_id.to_string();
                    let cents = magnitude.as_i64();
                    let signed = match split.transaction_type.as_deref() {
                        Some("withdrawal") => -cents.abs(),
                        Some("deposit") => cents.abs(),
                        _ if split.source_id.as_deref() == Some(account_id_str.as_str()) => {
                            -cents.abs()
                        }
                        _ => cents,
                    };
                    let ff3_id = group.id.parse::<i64>().map_err(|e| {
                        ApplicationError::Parse(format!("invalid FF3 transaction id: {e}"))
                    })?;
                    all.push(Ff3TransactionDto {
                        ff3_id,
                        date,
                        amount_cents: AmountCents::new(signed),
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
            // FF3 requires a positive amount; direction is conveyed by
            // type + source/destination accounts, never by the sign.
            "amount": Self::format_decimal(AmountCents::new(creation.amount.as_i64().abs())),
            "date": format!("{}T00:00:00+00:00", creation.date.format("%Y-%m-%d")),
            "description": creation.description,
        });
        let obj = attrs.as_object_mut().unwrap();
        // Optional counterparty (RF-08 v0.8): an absent side is not sent and
        // Firefly III applies its default handling for the movement.
        if let Some(src) = creation.source_ff3_id {
            obj.insert("source_id".into(), json!(src.to_string()));
        }
        if let Some(dst) = creation.destination_ff3_id {
            obj.insert("destination_id".into(), json!(dst.to_string()));
        }
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

        // FF3 transactions endpoint does NOT use the JSON-API data/attributes
        // wrapper on requests: TransactionStore has `transactions` at top level.
        let payload = json!({ "transactions": [attrs] });
        eprintln!("[FF3 create_transaction] payload: {payload}");
        let url = format!("{}/api/v1/transactions", self.base_url);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&payload)
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
            return Ok(());
        }
        let text = resp.text().await.unwrap_or_else(|_| "<no body>".into());
        // Already deleted (or never existed) in FF3: surface as NotFound
        // so the API answers 404 (RF-15 "already deleted" exception).
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(ApplicationError::NotFound(format!(
                "FF3 transaction {ff3_id} not found in Firefly III: {text}"
            )));
        }
        Err(ApplicationError::FireflyClient(format!(
            "FF3 returned {status}: {text}"
        )))
    }

    async fn tags(&self) -> Result<Vec<String>, ApplicationError> {
        self.paged_names("/api/v1/tags", |a| a.tag.clone()).await
    }

    async fn categories(&self) -> Result<Vec<String>, ApplicationError> {
        self.paged_names("/api/v1/categories", |a| a.name.clone())
            .await
    }

    async fn budgets(&self) -> Result<Vec<String>, ApplicationError> {
        self.paged_names("/api/v1/budgets", |a| a.name.clone())
            .await
    }

    async fn attach_to_transaction(
        &self,
        journal_ff3_id: i64,
        filename: &str,
        content_base64: &str,
    ) -> Result<(), ApplicationError> {
        use base64::Engine as _;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(content_base64)
            .map_err(|e| ApplicationError::Parse(format!("invalid attachment base64: {e}")))?;

        // Step 1: create attachment metadata.
        let url = format!("{}/api/v1/attachments", self.base_url);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "filename": filename,
                "attachable_type": "TransactionJournal",
                "attachable_id": journal_ff3_id.to_string(),
            }))
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
        let created: AttachmentCreateResponse = resp
            .json()
            .await
            .map_err(|e| ApplicationError::FireflyClient(e.to_string()))?;
        let attachment_id = created.data.id;

        // Step 2: upload file content.
        let url = format!(
            "{}/api/v1/attachments/{attachment_id}/upload",
            self.base_url
        );
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/octet-stream")
            .body(bytes)
            .send()
            .await
            .map_err(|e| ApplicationError::FireflyClient(e.to_string()))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_else(|_| "<no body>".into());
            Err(ApplicationError::FireflyClient(format!(
                "FF3 returned {}: {}",
                status, text
            )))
        }
    }
}
