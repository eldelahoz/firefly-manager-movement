//! `MysqlDeletedAuditRepository` — implements `DeletedAuditRepository`.

use async_trait::async_trait;
use diesel_async::RunQueryDsl;

use ffm_application::dtos::NewDeletedAudit;
use ffm_application::error::ApplicationError;
use ffm_application::ports::DeletedAuditRepository;
use ffm_domain::ids::DeletedAuditId;

use crate::models::NewDeletedAuditModel;
use crate::persistence::pool::DbPool;
use crate::schema::deleted_audit;

pub struct MysqlDeletedAuditRepository {
    pool: DbPool,
}

impl MysqlDeletedAuditRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

fn map_diesel_err(e: diesel::result::Error) -> ApplicationError {
    use diesel::result::Error as E;
    match e {
        E::NotFound => ApplicationError::NotFound("deleted_audit".into()),
        other => ApplicationError::Database(other.to_string()),
    }
}

#[async_trait]
impl DeletedAuditRepository for MysqlDeletedAuditRepository {
    async fn insert(&self, audit: NewDeletedAudit) -> Result<DeletedAuditId, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        let model = NewDeletedAuditModel::from(audit);
        diesel::insert_into(deleted_audit::table)
            .values(&model)
            .execute(&mut conn)
            .await
            .map_err(map_diesel_err)?;
        diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
            "LAST_INSERT_ID()",
        ))
        .get_result::<i64>(&mut conn)
        .await
        .map(DeletedAuditId::new)
        .map_err(map_diesel_err)
    }
}
