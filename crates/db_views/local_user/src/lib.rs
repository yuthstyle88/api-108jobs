use app_108jobs_db::source::{local_user::LocalUser, person::Person};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use {
  app_108jobs_db::utils::queries::creator_home_banned,
  diesel::{Queryable, Selectable},
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A local user view.
#[serde(rename_all = "camelCase")]
pub struct LocalUserView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_user: LocalUser,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person: Person,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_home_banned()
    )
  )]
  pub banned: bool,
}
