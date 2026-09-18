//! `MysqlAppSettingsRepository` - single-row (id = 1) settings storage.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use ffm_application::dtos::NewAppSettings;
use ffm_application::error::ApplicationError;
use ffm_application::ports::AppSettingsRepository;
use ffm_domain::entities::AppSettings;

use crate::models::AppSettingsModel;
use crate::persistence::pool::DbPool;
use crate::schema::app_settings;

pub struct MysqlAppSettingsRepository {
    pool: DbPool,
}

impl MysqlAppSettingsRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AppSettingsRepository for MysqlAppSettingsRepository {
    async fn get(&self) -> Result<Option<AppSettings>, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        match app_settings::table
            .filter(app_settings::id.eq(1))
            .first::<AppSettingsModel>(&mut conn)
            .await
        {
            Ok(m) => Ok(Some(m.into())),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(ApplicationError::Database(e.to_string())),
        }
    }

    async fn save(&self, settings: NewAppSettings) -> Result<AppSettings, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        diesel::sql_query(
            "INSERT INTO app_settings (id, ff3_url, ff3_token, default_creation_tag, enable_origin_trace, default_date_format) \
             VALUES (1, ?, ?, ?, ?, ?) \
             ON DUPLICATE KEY UPDATE ff3_url = VALUES(ff3_url), ff3_token = VALUES(ff3_token), \
             default_creation_tag = VALUES(default_creation_tag), \
             enable_origin_trace = VALUES(enable_origin_trace), \
             default_date_format = VALUES(default_date_format), updated_at = CURRENT_TIMESTAMP",
        )
        .bind::<diesel::sql_types::Varchar, _>(&settings.ff3_url)
        .bind::<diesel::sql_types::Varchar, _>(&settings.ff3_token)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Varchar>, _>(
            &settings.default_creation_tag,
        )
        .bind::<diesel::sql_types::Bool, _>(&settings.enable_origin_trace)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Varchar>, _>(
            &settings.default_date_format,
        )
        .execute(&mut conn)
        .await
        .map_err(|e| ApplicationError::Database(e.to_string()))?;
        self.get()
            .await?
            .ok_or_else(|| ApplicationError::Database("app_settings row missing after save".into()))
    }
}
