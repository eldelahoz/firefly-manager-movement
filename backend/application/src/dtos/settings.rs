//! Settings DTOs (RF-19). `SettingsView` is the frontend-facing shape:
//! the token is never included, only whether one is stored.

/// New/updated settings to persist (backend-side only).
#[derive(Debug, Clone)]
pub struct NewAppSettings {
    pub ff3_url: String,
    pub ff3_token: String,
    pub default_creation_tag: Option<String>,
}

/// Public view of the settings: no token value (NFR-02).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsView {
    pub ff3_url: String,
    pub has_token: bool,
    pub default_creation_tag: Option<String>,
}
