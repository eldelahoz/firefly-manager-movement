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
    /// Existing counterparty account (external FF3 id, must be cached).
    /// Both this and `counterparty_name` absent = Cash (RF-08 v0.10).
    pub counterparty_ff3_id: Option<i64>,
    /// New expense (money out) or revenue (money in) account, created by name.
    pub counterparty_name: Option<String>,
    /// Cash chosen explicitly: booked like no counterparty, but learned
    /// (RF-08 v0.11). Implicit Cash (nothing sent) teaches nothing.
    pub counterparty_cash: bool,
    pub description: Option<String>,
    pub category: Option<String>,
    pub budget: Option<String>,
    /// `None` = default creation tag (RF-19); `Some(v)` = exactly `v`, even empty (RF-08).
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,
    pub attachment: Option<AttachmentPayload>,
}

/// Input for the apply-missing use case.
#[derive(Debug, Clone)]
pub struct ApplyMissingInput {
    pub batch_id: BatchId,
    pub movements: Vec<MovementApply>,
}
