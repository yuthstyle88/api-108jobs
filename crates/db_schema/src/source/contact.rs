use crate::newtypes::{ContactId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::contact;
use std::convert::TryFrom;
use lemmy_utils::error::{FastJobError, FastJobErrorType};
use lemmy_utils::utils::validation::{is_valid_email, is_valid_phone};

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = contact))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct Contact {
  pub id: ContactId,
  pub phone: Option<String>,
  pub email: Option<String>,
  pub secondary_email: Option<String>,
  pub line_id: Option<String>,
  pub facebook: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = contact))]
pub struct ContactInsertForm {
  pub phone: Option<String>,
  pub email: Option<String>,
  pub secondary_email: Option<String>,
  pub line_id: Option<String>,
  pub facebook: Option<String>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = contact))]
pub struct ContactUpdateForm {
  pub phone: Option<String>,
  pub email: Option<String>,
  pub secondary_email: Option<String>,
  pub line_id: Option<String>,
  pub facebook: Option<String>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct ContactForm {
  pub phone: Option<String>,
  pub email: Option<String>,
  pub secondary_email: Option<String>,
  pub line_id: Option<String>,
  pub facebook: Option<String>,
}

impl TryFrom<ContactForm> for ContactUpdateForm {
  type Error = FastJobError;

  fn try_from(form: ContactForm) -> Result<Self, Self::Error> {
    // Validate that at least one contact method is provided
    let _ = validate_contact(&form)?;

    Ok(Self {
      phone: form.phone,
      email: form.email,
      secondary_email: form.secondary_email,
      line_id: form.line_id,
      facebook: form.facebook,
      updated_at: Some(Some(Utc::now())),
    })
  }
}
fn validate_contact(form: &ContactForm) -> Result<(), FastJobError> {
  let has_contact = [
    &form.phone,
    &form.email,
    &form.line_id,
    &form.facebook,
  ].iter().any(|f| f.is_some());

  if !has_contact {
    return Err(FastJobErrorType::ValidationError("At least one contact method must be provided".to_string()).into());
  }

  // Validate email format if provided
  if let Some(email) = &form.email {
    if !is_valid_email(email) {
      return Err(FastJobErrorType::InvalidEmail.into());
    }
  }

  // Validate secondary email format if provided
  if let Some(secondary_email) = &form.secondary_email {
    if !is_valid_email(secondary_email) {
      return Err(FastJobErrorType::InvalidEmail.into());
    }
  }

  // Validate phone format if provided
  if let Some(phone) = &form.phone {
    if !is_valid_phone(phone) {
      return Err(FastJobErrorType::ValidationError("Invalid phone format".to_string()).into());
    }
  }
  Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactResponse {
  pub contact: Contact,
}