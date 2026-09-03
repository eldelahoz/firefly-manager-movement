use chrono::NaiveDateTime;
use diesel::prelude::*;

use ffm_application::dtos::NewFf3Account;
use ffm_domain::entities::Ff3Account;
use ffm_domain::ids::Ff3AccountId;

use crate::schema::ff3_accounts;

#[derive(Queryable, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = ff3_accounts)]
pub struct Ff3AccountModel {
    pub id: i64,
    pub ff3_id: i32,
    pub name: String,
    pub type_: String,
    pub currency: String,
    pub iban: Option<String>,
    pub account_number: Option<String>,
    pub synced_at: NaiveDateTime,
}

impl From<Ff3AccountModel> for Ff3Account {
    fn from(m: Ff3AccountModel) -> Self {
        Self {
            id: Ff3AccountId::new(m.id),
            ff3_id: m.ff3_id as i64,
            name: m.name,
            account_type: m.type_,
            currency: m.currency,
            iban: m.iban,
            account_number: m.account_number,
            synced_at: m.synced_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = ff3_accounts)]
pub struct NewFf3AccountModel {
    pub ff3_id: i32,
    pub name: String,
    pub type_: String,
    pub currency: String,
    pub iban: Option<String>,
    pub account_number: Option<String>,
}

impl From<NewFf3Account> for NewFf3AccountModel {
    fn from(d: NewFf3Account) -> Self {
        Self {
            ff3_id: d.ff3_id as i32,
            name: d.name,
            type_: d.account_type,
            currency: d.currency,
            iban: d.iban,
            account_number: d.account_number,
        }
    }
}
