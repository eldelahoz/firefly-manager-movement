//! App settings handlers (RF-19). The token is write-only (NFR-02).

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::ApiError;
use crate::state::AppState;
use ffm_application::dtos::SettingsView;
use ffm_application::use_cases::app_settings::{
    GetAppSettings, SaveAppSettings, SaveAppSettingsInput, VerifyFireflyConnection,
};

#[derive(Serialize, ToSchema)]
pub struct SettingsResponse {
    pub ff3_url: String,
    pub has_token: bool,
    pub default_creation_tag: Option<String>,
    pub enable_origin_trace: bool,
    /// Global default applied when the import mapping carries no explicit format.
    ///
    /// `null` = no global default: parsing falls back to the legacy ordered
    /// auto-try. (RF-02 v0.8)
    pub default_date_format: Option<String>,
}

impl From<SettingsView> for SettingsResponse {
    fn from(v: SettingsView) -> Self {
        Self {
            ff3_url: v.ff3_url,
            has_token: v.has_token,
            default_creation_tag: v.default_creation_tag,
            enable_origin_trace: v.enable_origin_trace,
            default_date_format: v.default_date_format,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct SaveSettingsRequest {
    pub ff3_url: String,
    /// New FF3 API token to store.
    ///
    /// Empty/absent = keep the stored token.
    #[serde(default)]
    pub ff3_token: Option<String>,
    /// null or empty string clears the default tag.
    pub default_creation_tag: Option<String>,
    /// When false, origin trace notes are not appended to created transactions.
    #[serde(default = "default_true")]
    pub enable_origin_trace: Option<bool>,
    /// Global default applied when the import mapping carries no explicit format.
    ///
    /// `null`/empty/absent clears the stored default; unknown values yield
    /// 400. (RF-02 v0.8)
    #[serde(default)]
    pub default_date_format: Option<String>,
}

fn default_true() -> Option<bool> {
    Some(true)
}

#[derive(Deserialize, ToSchema)]
pub struct VerifySettingsRequest {
    pub ff3_url: String,
    pub ff3_token: String,
}

#[derive(Serialize, ToSchema)]
pub struct VerifySettingsResponse {
    pub version: String,
}

#[utoipa::path(
    get,
    path = "/api/settings",
    responses(
        (status = 200, description = "App settings (token never returned)", body = SettingsResponse)
    ),
    tag = "settings"
)]
pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<SettingsResponse>, ApiError> {
    let view = GetAppSettings::new(state.app_settings.clone(), state.env_defaults.clone())
        .execute()
        .await
        .map_err(ApiError)?;
    Ok(Json(view.into()))
}

#[utoipa::path(
    put,
    path = "/api/settings",
    request_body = SaveSettingsRequest,
    responses(
        (status = 200, description = "Settings saved; connection reconfigured", body = SettingsResponse),
        (status = 400, description = "Invalid URL"),
        (status = 500, description = "No token available"),
    ),
    tag = "settings"
)]
pub async fn save_settings(
    State(state): State<AppState>,
    Json(body): Json<SaveSettingsRequest>,
) -> Result<Json<SettingsResponse>, ApiError> {
    let _guard = state.settings_lock.lock().await;
    let view = SaveAppSettings::new(
        state.app_settings.clone(),
        state.firefly_manager.clone(),
        state.env_defaults.clone(),
    )
    .execute(SaveAppSettingsInput {
        ff3_url: body.ff3_url,
        ff3_token: body.ff3_token,
        default_creation_tag: body.default_creation_tag,
        enable_origin_trace: body.enable_origin_trace.unwrap_or(true),
        default_date_format: body.default_date_format,
    })
    .await
    .map_err(ApiError)?;
    Ok(Json(view.into()))
}

#[utoipa::path(
    post,
    path = "/api/settings/verify",
    request_body = VerifySettingsRequest,
    responses(
        (status = 200, description = "Connection OK", body = VerifySettingsResponse),
        (status = 400, description = "Invalid URL or empty token"),
        (status = 502, description = "Firefly III unreachable / bad credentials"),
    ),
    tag = "settings"
)]
pub async fn verify_settings(
    State(state): State<AppState>,
    Json(body): Json<VerifySettingsRequest>,
) -> Result<Json<VerifySettingsResponse>, ApiError> {
    let about = VerifyFireflyConnection::new(state.firefly_factory.clone())
        .execute(body.ff3_url, body.ff3_token)
        .await
        .map_err(ApiError)?;
    Ok(Json(VerifySettingsResponse {
        version: about.version,
    }))
}
