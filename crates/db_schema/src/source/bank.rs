use crate::newtypes::BankId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::banks;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = banks))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct Bank {
  pub id: BankId,
  pub name: String,
  pub country_id: String,
  pub bank_code: Option<String>,
  pub swift_code: Option<String>,
  pub is_active: Option<bool>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = banks))]
pub struct BankInsertForm {
  pub name: String,
  pub country_id: String,
  pub bank_code: Option<String>,
  pub swift_code: Option<String>,
  pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = banks))]
pub struct BankUpdateForm {
  pub name: Option<String>,
  pub country_id: Option<String>,
  pub bank_code: Option<Option<String>>,
  pub swift_code: Option<Option<String>>,
  pub is_active: Option<bool>,
  pub updated_at: Option<DateTime<Utc>>,
}