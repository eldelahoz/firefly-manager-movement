use std::collections::HashSet;

use async_trait::async_trait;

use ffm_domain::ids::DeletedAuditId;

use crate::dtos::NewDeletedAudit;
use crate::error::ApplicationError;

#[async_trait]
pub trait DeletedAuditRepository: Send + Sync {
    async fn insert(&self, audit: NewDeletedAudit) -> Result<DeletedAuditId, ApplicationError>;

    /// FF3 ids of transactions already deleted (RF-15): used to remove
    /// them from the extras list in results.
    async fn deleted_ff3_ids(&self) -> Result<HashSet<i64>, ApplicationError>;
}
