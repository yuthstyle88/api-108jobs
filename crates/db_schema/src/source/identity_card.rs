use crate::newtypes::{AddressId, IdentityCardId};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::identity_card;
use std::convert::TryFrom;
use lemmy_utils::error::{FastJobError, FastJobErrorType};
use lemmy_utils::utils::validation::is_valid_issued_and_expiry;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = identity_card))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct IdentityCard {
  pub id: IdentityCardId,
  pub address_id: Option<AddressId>,
  pub id_number: String,
  pub issued_date: Option<NaiveDate>,
  pub expiry_date: Option<NaiveDate>,
  pub full_name: Option<String>,
  pub date_of_birth: Option<NaiveDate>,
  pub nationality: Option<String>,
  pub is_verified: bool,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = identity_card))]
pub struct IdentityCardInsertForm {
  pub address_id: Option<AddressId>,
  pub id_number: String,
  pub issued_date: Option<NaiveDate>,
  pub expiry_date: Option<NaiveDate>,
  pub full_name: Option<String>,
  pub date_of_birth: Option<NaiveDate>,
  pub nationality: Option<String>,
  pub is_verified: Option<bool>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = identity_card))]
pub struct IdentityCardUpdateForm {
  pub address_id: Option<AddressId>,
  pub id_number: Option<String>,
  pub issued_date: Option<NaiveDate>,
  pub expiry_date: Option<NaiveDate>,
  pub full_name: Option<String>,
  pub date_of_birth: Option<NaiveDate>,
  pub nationality: Option<String>,
  pub is_verified: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityCardForm {
  pub address_id: Option<AddressId>,
  pub id_number: Option<String>,
  pub issued_date: Option<NaiveDate>,
  pub expiry_date: Option<NaiveDate>,
  pub full_name: Option<String>,
  pub date_of_birth: Option<NaiveDate>,
  pub nationality: Option<String>,
  pub is_verified: Option<bool>,
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityCardResponse {
  pub identity_card: IdentityCard,
}

impl TryFrom<IdentityCardForm> for IdentityCardUpdateForm {
  type Error = FastJobError;

  fn try_from(form: IdentityCardForm) -> Result<Self, Self::Error> {
    // Validate that id_number is not empty if provided
    if let Some(id_number) = &form.id_number {
      if id_number.trim().is_empty() {
        return Err(FastJobErrorType::ValidationError("ID number cannot be empty".to_string()).into());
      }
    }
    
    // Validate expiry date is after issued date if both are provided
    if  !is_valid_issued_and_expiry(form.issued_date, form.expiry_date){
      return Err(FastJobErrorType::InvalidIssueAndExpire.into());
    }

    Ok(Self {
      address_id: form.address_id,
      id_number: form.id_number,
      issued_date: form.issued_date,
      expiry_date: form.expiry_date,
      full_name: form.full_name,
      date_of_birth: form.date_of_birth,
      nationality: form.nationality,
      is_verified: form.is_verified,
    })
  }
}
