use ffm_domain::ids::{BatchId, MovementId};

/// Binary payload for a file attachment to upload to Firefly III.
#[derive(Debug, Clone)]
pub struct AttachmentPayload {
    pub filename: String,
    pub content_base64: String,
}

/// Per-movement customization for the apply operation.
#[derive(Debug, Clone)]
pub struct MovementApply {
    pub movement_id: MovementId,
    pub counterparty_ff3_id: i64,
    pub description: Option<String>,
    pub category: Option<String>,
    pub budget: Option<String>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub attachment: Option<AttachmentPayload>,
}

/// Input for the apply-missing use case.
#[derive(Debug, Clone)]
pub struct ApplyMissingInput {
    pub batch_id: BatchId,
    pub movements: Vec<MovementApply>,
}
