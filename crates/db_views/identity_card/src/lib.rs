use lemmy_db_schema::source::identity_card::IdentityCard;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use {
  diesel::{helper_types::Nullable, NullableExpressionMethods, Queryable, Selectable},
  lemmy_db_schema::utils::{functions::coalesce, queries::creator_banned},
  lemmy_db_schema_file::schema::local_user,
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// An identity card view.
#[serde(rename_all = "camelCase")]
pub struct IdentityCardView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub identity_card: IdentityCard,
}
