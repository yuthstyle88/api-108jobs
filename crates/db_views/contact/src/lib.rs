use lemmy_db_schema::source::contact::Contact;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use {
  diesel::{NullableExpressionMethods, Queryable, Selectable},
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A contact view.
#[serde(rename_all = "camelCase")]
pub struct ContactView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub contact: Contact,
}
