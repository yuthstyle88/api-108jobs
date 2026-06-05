use crate::PersonView;
use app_108jobs_db_schema::source::person::Person;
use app_108jobs_db_schema::{newtypes::PersonId, source::site::Site};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Adds an admin to a site.
pub struct AddAdminRequest {
  pub person_id: PersonId,
  pub added: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The response of current admins.
#[serde(rename_all = "camelCase")]
pub struct AddAdminResponse {
  pub admins: Vec<PersonView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Ban a person from the site.
pub struct BanPerson {
  pub person_id: PersonId,
  pub ban: bool,
  /// Optionally remove or restore all their data. Useful for new troll accounts.
  /// If ban is true, then this means remove. If ban is false, it means restore.
  pub remove_or_restore_data: Option<bool>,
  pub reason: Option<String>,
  /// A time that the ban will expire, in unix epoch seconds.
  ///
  /// An i64 unix timestamp is used for a simpler API client implementation.
  pub expires_at: Option<i64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Ban a person from the site.
pub struct BanPersonRequest {
  pub person_id: PersonId,
  pub ban: bool,
  /// Optionally remove or restore all their data. Useful for new troll accounts.
  /// If ban is true, then this means remove. If ban is false, it means restore.
  pub remove_or_restore_data: Option<bool>,
  pub reason: Option<String>,
  /// A time that the ban will expire, in unix epoch seconds.
  ///
  /// An i64 unix timestamp is used for a simpler API client implementation.
  pub expires_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A response for a banned person.
#[serde(rename_all = "camelCase")]
pub struct BanPersonResponse {
  pub person_view: PersonView,
  pub banned: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Block a person.
pub struct BlockPerson {
  pub person_id: PersonId,
  pub block: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The response for a person block.
#[serde(rename_all = "camelCase")]
pub struct BlockPersonResponse {
  pub person_view: PersonView,
  pub blocked: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Gets a person's details.
///
/// Either person_id, or username are required.
pub struct GetPersonDetails {
  pub person_id: Option<PersonId>,
  /// Example: dessalines , or dessalines@xyz.tld
  pub username: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A person's details response.
#[serde(rename_all = "camelCase")]
pub struct GetPersonDetailsResponse {
  pub person_view: PersonView,
  pub site: Option<Site>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Purges a person from the database. This will delete all content attached to that person.
pub struct PurgePerson {
  pub person_id: PersonId,
  pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Make a note for a person.
///
/// An empty string deletes the note.
pub struct NotePerson {
  pub person_id: PersonId,
  pub note: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct VisitProfileResponse {
  pub profile: Person,
}
