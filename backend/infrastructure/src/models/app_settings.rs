use chrono::NaiveDateTime;
use diesel::prelude::*;

use ffm_application::dtos::NewAppSettings;
use ffm_domain::entities::AppSettings;
use ffm_domain::value_objects::DateFormat;

use crate::schema::app_settings;

#[derive(Queryable, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = app_settings)]
pub struct AppSettingsModel {
    pub id: i32,
    pub ff3_url: String,
    pub ff3_token: String,
    pub default_creation_tag: Option<String>,
    pub updated_at: NaiveDateTime,
    pub enable_origin_trace: bool,
    pub default_date_format: Option<String>,
}

impl From<AppSettingsModel> for AppSettings {
    fn from(m: AppSettingsModel) -> Self {
        Self {
            ff3_url: m.ff3_url,
            ff3_token: m.ff3_token,
            default_creation_tag: m.default_creation_tag,
            enable_origin_trace: m.enable_origin_trace,
            default_date_format: m
                .default_date_format
                .and_then(|s| s.parse::<DateFormat>().ok()),
            updated_at: m.updated_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = app_settings)]
pub struct NewAppSettingsModel {
    pub id: i32,
    pub ff3_url: String,
    pub ff3_token: String,
    pub default_creation_tag: Option<String>,
    pub enable_origin_trace: bool,
    pub default_date_format: Option<String>,
}

impl From<NewAppSettings> for NewAppSettingsModel {
    fn from(d: NewAppSettings) -> Self {
        Self {
            id: 1,
            ff3_url: d.ff3_url,
            ff3_token: d.ff3_token,
            default_creation_tag: d.default_creation_tag,
            enable_origin_trace: d.enable_origin_trace,
            default_date_format: d.default_date_format,
        }
    }
}
