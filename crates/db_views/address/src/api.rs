use crate::AddressView;
use lemmy_db_schema::newtypes::LocalUserId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Create or update address information.
pub struct UpsertAddress {
  pub address_line1: String,
  pub address_line2: Option<String>,
  pub subdistrict: String,
  pub district: String,
  pub province: String,
  pub postal_code: String,
  pub country_id: String,
  pub is_default: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Create or update address information.
pub struct UpsertAddressRequest {
  pub address_line1: Option<String>,
  pub address_line2: Option<String>,
  pub subdistrict: Option<String>,
  pub district: Option<String>,
  pub province: Option<String>,
  pub postal_code: Option<String>,
  pub country_id: Option<String>,
  pub is_default: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The response for an address operation.
#[serde(rename_all = "camelCase")]
pub struct AddressResponse {
  pub address_view: AddressView,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Gets address details.
pub struct GetAddressDetails {
  pub local_user_id: LocalUserId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A address details response.
#[serde(rename_all = "camelCase")]
pub struct GetAddressDetailsResponse {
  pub address_view: Option<AddressView>,
}
