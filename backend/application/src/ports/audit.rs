use async_trait::async_trait;

use ffm_domain::ids::DeletedAuditId;

use crate::dtos::NewDeletedAudit;
use crate::error::ApplicationError;

#[async_trait]
pub trait DeletedAuditRepository: Send + Sync {
    async fn insert(&self, audit: NewDeletedAudit) -> Result<DeletedAuditId, ApplicationError>;
}
