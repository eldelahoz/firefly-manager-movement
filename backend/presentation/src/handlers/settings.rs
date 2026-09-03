//! App settings handlers (RF-19). The token is write-only (NFR-02).

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use ffm_application::dtos::SettingsView;
use ffm_application::use_cases::app_settings::{
    GetAppSettings, SaveAppSettings, SaveAppSettingsInput, VerifyFireflyConnection,
};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct SettingsResponse {
    pub ff3_url: String,
    pub has_token: bool,
    pub default_creation_tag: Option<String>,
}

impl From<SettingsView> for SettingsResponse {
    fn from(v: SettingsView) -> Self {
        Self {
            ff3_url: v.ff3_url,
            has_token: v.has_token,
            default_creation_tag: v.default_creation_tag,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct SaveSettingsRequest {
    pub ff3_url: String,
    /// Optional on update: empty/absent keeps the stored token.
    #[serde(default)]
    pub ff3_token: Option<String>,
    /// null or empty string clears the default tag.
    pub default_creation_tag: Option<String>,
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
    let view = GetAppSettings::new(state.app_settings.clone())
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
    let view = SaveAppSettings::new(state.app_settings.clone(), state.firefly_manager.clone())
        .execute(SaveAppSettingsInput {
            ff3_url: body.ff3_url,
            ff3_token: body.ff3_token,
            default_creation_tag: body.default_creation_tag,
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
