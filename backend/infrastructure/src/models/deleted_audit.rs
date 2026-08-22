use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde_json::Value as JsonValue;

use ffm_application::dtos::NewDeletedAudit;
use ffm_domain::entities::DeletedAudit;
use ffm_domain::ids::DeletedAuditId;

use crate::schema::deleted_audit;

#[derive(Queryable, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = deleted_audit)]
pub struct DeletedAuditModel {
    pub id: i64,
    pub ff3_id: i64,
    pub snapshot_json: JsonValue,
    pub deleted_at: NaiveDateTime,
}

impl From<DeletedAuditModel> for DeletedAudit {
    fn from(m: DeletedAuditModel) -> Self {
        Self {
            id: DeletedAuditId::new(m.id),
            ff3_id: m.ff3_id,
            snapshot_json: m.snapshot_json.to_string(),
            deleted_at: m.deleted_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = deleted_audit)]
pub struct NewDeletedAuditModel {
    pub ff3_id: i64,
    pub snapshot_json: JsonValue,
}

impl From<NewDeletedAudit> for NewDeletedAuditModel {
    fn from(d: NewDeletedAudit) -> Self {
        Self {
            ff3_id: d.ff3_id,
            snapshot_json: serde_json::from_str(&d.snapshot_json).unwrap_or(JsonValue::Null),
        }
    }
}
