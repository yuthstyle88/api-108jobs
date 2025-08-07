use crate::ContactView;
use lemmy_db_schema::newtypes::LocalUserId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Create or update contact information.
pub struct CreateOrUpdateContact {
  pub local_user_id: LocalUserId,
  pub phone: Option<String>,
  pub email: Option<String>,
  pub secondary_email: Option<String>,
  pub line_id: Option<String>,
  pub facebook: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The response for a contact operation.
#[serde(rename_all = "camelCase")]
pub struct ContactResponse {
  pub contact_view: ContactView,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Gets contact details.
pub struct GetContactDetails {
  pub local_user_id: LocalUserId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A contact details response.
#[serde(rename_all = "camelCase")]
pub struct GetContactDetailsResponse {
  pub contact_view: Option<ContactView>,
}