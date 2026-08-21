//! `MysqlFf3AccountRepository` — implements `Ff3AccountRepository`.
//! The upsert uses raw SQL with `ON DUPLICATE KEY UPDATE` (MySQL has no
//! Postgres `ON CONFLICT`). The `type` column is named `type_` in diesel.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use ffm_application::dtos::NewFf3Account;
use ffm_application::error::ApplicationError;
use ffm_application::ports::Ff3AccountRepository;
use ffm_domain::entities::Ff3Account;

use crate::models::Ff3AccountModel;
use crate::persistence::pool::DbPool;
use crate::schema::ff3_accounts;

pub struct MysqlFf3AccountRepository {
    pool: DbPool,
}

impl MysqlFf3AccountRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Ff3AccountRepository for MysqlFf3AccountRepository {
    async fn upsert_batch(&self, accounts: Vec<NewFf3Account>) -> Result<(), ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        for account in accounts {
            diesel::sql_query(
                "INSERT INTO ff3_accounts (ff3_id, name, type, currency) VALUES (?, ?, ?, ?) \
                 ON DUPLICATE KEY UPDATE name = VALUES(name), type = VALUES(type), \
                 currency = VALUES(currency), synced_at = CURRENT_TIMESTAMP",
            )
            .bind::<diesel::sql_types::Integer, _>(account.ff3_id as i32)
            .bind::<diesel::sql_types::Varchar, _>(&account.name)
            .bind::<diesel::sql_types::Varchar, _>(&account.account_type)
            .bind::<diesel::sql_types::Varchar, _>(&account.currency)
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        }
        Ok(())
    }

    async fn by_ff3_id(&self, ff3_id: i64) -> Result<Option<Ff3Account>, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        match ff3_accounts::table
            .filter(ff3_accounts::ff3_id.eq(ff3_id as i32))
            .first::<Ff3AccountModel>(&mut conn)
            .await
        {
            Ok(a) => Ok(Some(a.into())),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(ApplicationError::Database(e.to_string())),
        }
    }

    async fn all_asset(&self) -> Result<Vec<Ff3Account>, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        ff3_accounts::table
            .filter(ff3_accounts::type_.eq("asset"))
            .order(ff3_accounts::name.asc())
            .load::<Ff3AccountModel>(&mut conn)
            .await
            .map(|v| v.into_iter().map(Into::into).collect())
            .map_err(|e| ApplicationError::Database(e.to_string()))
    }
}
