use chrono::NaiveDateTime;
use diesel::prelude::*;

use ffm_application::dtos::NewCreation;
use ffm_domain::entities::Creation;
use ffm_domain::ids::{CreationId, MovementId};

use crate::schema::creations;

#[derive(Queryable, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = creations)]
pub struct CreationModel {
    pub id: i64,
    pub movement_id: i64,
    pub ff3_id: i64,
    pub created_at: NaiveDateTime,
}

impl From<CreationModel> for Creation {
    fn from(m: CreationModel) -> Self {
        Self {
            id: CreationId::new(m.id),
            movement_id: MovementId::new(m.movement_id),
            ff3_id: m.ff3_id,
            created_at: m.created_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = creations)]
pub struct NewCreationModel {
    pub movement_id: i64,
    pub ff3_id: i64,
}

impl From<NewCreation> for NewCreationModel {
    fn from(d: NewCreation) -> Self {
        Self {
            movement_id: d.movement_id.as_i64(),
            ff3_id: d.ff3_id,
        }
    }
}
