use lemmy_db_schema::source::wallet::Wallet;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use diesel::{Queryable, Selectable};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A wallet view.
#[serde(rename_all = "camelCase")]
pub struct WalletView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub wallet: Wallet,
}