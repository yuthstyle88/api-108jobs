use crate::IdentityCardView;
use chrono::NaiveDate;
use lemmy_db_schema::newtypes::{AddressId, LocalUserId};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Create or update identity card information.
pub struct UpsertIDCard {
  pub address_id: AddressId,
  pub id_number: String,
  pub issued_date: NaiveDate,
  pub expiry_date: NaiveDate,
  pub full_name: String,
  pub date_of_birth: NaiveDate,
  pub nationality: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Create or update identity card information.
pub struct UpsertIDCardRequest {
  pub address_id: Option<AddressId>,
  pub id_number: Option<String>,
  pub issued_date: Option<NaiveDate>,
  pub expiry_date: Option<NaiveDate>,
  pub full_name: Option<String>,
  pub date_of_birth: Option<NaiveDate>,
  pub nationality: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The response for an identity card operation.
#[serde(rename_all = "camelCase")]
pub struct IdentityCardResponse {
  pub identity_card_view: IdentityCardView,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Gets identity card details.
pub struct GetIdentityCardDetails {
  pub local_user_id: LocalUserId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// An identity card details response.
#[serde(rename_all = "camelCase")]
pub struct GetIdentityCardDetailsResponse {
  pub identity_card_view: Option<IdentityCardView>,
}
