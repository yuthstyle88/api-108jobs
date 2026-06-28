pub mod api;
mod impls;
pub mod ride_session_view;
pub mod validator;

use app_108jobs_db::source::{person::Person, rider::Rider};
#[cfg(feature = "full")]
use diesel::{Queryable, Selectable};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct RiderView {
  /// Base rider row
  #[cfg_attr(feature = "full", diesel(embed))]
  pub rider: Rider,

  /// Person profile
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person: Person,
}
