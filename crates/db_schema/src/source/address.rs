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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct AddressForm {
  pub address_line1: String,
  pub address_line2: Option<String>,
  pub subdistrict: Option<String>,
  pub district: String,
  pub province: String,
  pub postal_code: String,
  pub country_id: String,
  pub is_default: Option<bool>,
}

impl TryFrom<AddressForm> for AddressUpdateForm {
  type Error = FastJobError;

  fn try_from(form: AddressForm) -> Result<Self, Self::Error> {
    // Validate that required fields are not empty
    let _ = validate_address(&form)?;
    Ok(Self {
      address_line1: Some(form.address_line1),
      address_line2: Some(form.address_line2),
      subdistrict: Some(form.subdistrict),
      district: Some(form.district),
      province: Some(form.province),
      postal_code: Some(form.postal_code),
      country_id: Some(form.country_id),
      is_default: form.is_default,
    })
  }
}
fn validate_address(form: &AddressForm) -> Result<(), FastJobError> {
  if form.address_line1.trim().is_empty() {
    return Err(
      FastJobErrorType::ValidationError("Address line 1 cannot be empty".to_string()).into(),
    );
  }

  if form.subdistrict.is_none() {
    return Err(
      FastJobErrorType::ValidationError("Subdistrict cannot be empty".to_string()).into(),
    );
  }

  if form.district.trim().is_empty() {
    return Err(FastJobErrorType::ValidationError("District cannot be empty".to_string()).into());
  }

  if form.province.trim().is_empty() {
    return Err(FastJobErrorType::ValidationError("Province cannot be empty".to_string()).into());
  }

  if form.postal_code.trim().is_empty() {
    return Err(
      FastJobErrorType::ValidationError("Postal code cannot be empty".to_string()).into(),
    );
  }

  // Validate postal code format (assuming Thai postal code format of 5 digits)
  if !form.postal_code.chars().all(char::is_numeric) || form.postal_code.len() != 5 {
    return Err(FastJobErrorType::ValidationError("Invalid postal code format".to_string()).into());
  }
  Ok(())
}
