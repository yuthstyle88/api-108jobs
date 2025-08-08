use crate::RegistrationApplicationView;
use lemmy_db_schema::{
  newtypes::{PaginationCursor, PersonId, RegistrationApplicationId},
  sensitive::SensitiveString,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Approves a registration application.
pub struct ApproveRegistrationApplication {
  pub id: RegistrationApplicationId,
  pub approve: bool,
  pub deny_reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Gets a registration application for a person
pub struct GetRegistrationApplication {
  pub person_id: PersonId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Fetches a list of registration applications.
pub struct ListRegistrationApplications {
  /// Only shows the unread applications (IE those without an admin actor)
  pub unread_only: Option<bool>,
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The list of registration applications.
#[serde(rename_all = "camelCase")]
pub struct ListRegistrationApplicationsResponse {
  pub registration_applications: Vec<RegistrationApplicationView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Register / Sign up to lemmy.
#[serde(rename_all = "camelCase")]
pub struct Register {
  pub username: String,
  pub password: SensitiveString,
  pub self_promotion: Option<bool>,
  /// multilang is mandatory if multilang verification is enabled on the server
  pub email: Option<SensitiveString>,
  /// The UUID of the captcha item.
  pub captcha_uuid: Option<String>,
  /// Your captcha answer.
  pub captcha_answer: Option<String>,
  /// A form field to trick signup bots. Should be None.
  pub honeypot: Option<String>,
  /// An answer is mandatory if require application is enabled on the server
  pub answer: Option<String>,
  pub accepted_application: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct RegisterRequest {
  pub username: Option<String>,
  pub password: Option<SensitiveString>,
  pub password_verify: Option<SensitiveString>,
  pub self_promotion: Option<bool>,
  pub email: Option<SensitiveString>,
  pub captcha_uuid: Option<String>,
  pub captcha_answer: Option<String>,
  pub honeypot: Option<String>,
  pub answer: Option<String>,
  pub accepted_application: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct OAuthUserUpdateRequest {
  pub password: Option<SensitiveString>,
  pub password_verify: Option<SensitiveString>,
  pub email: Option<String>,
  pub terms_accepted: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The response of an action done to a registration application.
#[serde(rename_all = "camelCase")]
pub struct RegistrationApplicationResponse {
  pub registration_application: RegistrationApplicationView,
}
