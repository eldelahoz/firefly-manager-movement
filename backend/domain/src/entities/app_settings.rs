use chrono::NaiveDateTime;

/// App settings (RF-19): single row. `ff3_token` is write-only and must
/// never be serialized to the frontend (NFR-02).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSettings {
    pub ff3_url: String,
    pub ff3_token: String,
    pub default_creation_tag: Option<String>,
    pub enable_origin_trace: bool,
    pub updated_at: NaiveDateTime,
}
