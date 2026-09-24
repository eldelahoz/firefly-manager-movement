//! Settings DTOs (RF-19). `SettingsView` is the frontend-facing shape:
//! the token is never included, only whether one is stored.

/// New/updated settings to persist (backend-side only).
#[derive(Debug, Clone)]
pub struct NewAppSettings {
    pub ff3_url: String,
    pub ff3_token: String,
    pub default_creation_tag: Option<String>,
    pub enable_origin_trace: bool,
    /// Wire date-format string (lowercase, e.g. `dd/mm/yyyy`).
    ///
    /// Blank/`None` clears the stored default; unknown values are rejected
    /// with 400 before persisting. (RF-02 v0.8)
    pub default_date_format: Option<String>,
}

/// Public view of the settings: no token value (NFR-02).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsView {
    pub ff3_url: String,
    pub has_token: bool,
    pub default_creation_tag: Option<String>,
    pub enable_origin_trace: bool,
    /// Wire date-format string (lowercase, e.g. `dd/mm/yyyy`).
    ///
    /// `None` = no global default: parsing falls back to the legacy ordered
    /// auto-try. (RF-02 v0.8)
    pub default_date_format: Option<String>,
}
