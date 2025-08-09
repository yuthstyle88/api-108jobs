use crate::newtypes::AddressId;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::address;
use lemmy_utils::error::{FastJobError, FastJobErrorType};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::convert::TryFrom;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = address))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct Address {
  pub id: AddressId,
  pub address_line1: String,
  pub address_line2: Option<String>,
  pub subdistrict: Option<String>,
  pub district: String,
  pub province: String,
  pub postal_code: String,
  pub country_id: String,
  pub is_default: bool,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = address))]
pub struct AddressInsertForm {
  pub address_line1: String,
  pub address_line2: Option<String>,
  pub subdistrict: Option<String>,
  pub district: String,
  pub province: String,
  pub postal_code: String,
  #[new(default)]
  pub country_id: Option<String>,
  pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = address))]
pub struct AddressUpdateForm {
  pub address_line1: Option<String>,
  pub address_line2: Option<Option<String>>,
  pub subdistrict: Option<Option<String>>,
  pub district: Option<String>,
  pub province: Option<String>,
  pub postal_code: Option<String>,
  pub country_id: Option<String>,
  pub is_default: Option<bool>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
}
