use std::str::FromStr;

use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use serde_json::Value as JsonValue;

use ffm_application::dtos::NewMovement;
use ffm_domain::entities::Movement;
use ffm_domain::ids::{BatchId, MovementId};
use ffm_domain::value_objects::{AmountCents, MovementStatus};

use crate::schema::movements;

#[derive(Queryable, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = movements)]
pub struct MovementModel {
    pub id: i64,
    pub batch_id: i64,
    pub line_no: i32,
    pub date: NaiveDate,
    pub amount_cents: i64,
    pub description: String,
    pub raw_json: JsonValue,
    pub status: String,
    pub created_at: NaiveDateTime,
}

impl From<MovementModel> for Movement {
    fn from(m: MovementModel) -> Self {
        Self {
            id: MovementId::new(m.id),
            batch_id: BatchId::new(m.batch_id),
            line_no: m.line_no,
            date: m.date,
            amount_cents: AmountCents::new(m.amount_cents),
            description: m.description,
            raw_json: m.raw_json.to_string(),
            status: MovementStatus::from_str(&m.status).unwrap_or(MovementStatus::Imported),
            created_at: m.created_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = movements)]
pub struct NewMovementModel {
    pub batch_id: i64,
    pub line_no: i32,
    pub date: NaiveDate,
    pub amount_cents: i64,
    pub description: String,
    pub raw_json: JsonValue,
    pub status: String,
}

impl From<NewMovement> for NewMovementModel {
    fn from(d: NewMovement) -> Self {
        Self {
            batch_id: d.batch_id.as_i64(),
            line_no: d.line_no,
            date: d.date,
            amount_cents: d.amount_cents.as_i64(),
            description: d.description,
            raw_json: serde_json::from_str(&d.raw_json).unwrap_or(JsonValue::Null),
            status: d.status.as_str().to_string(),
        }
    }
}
