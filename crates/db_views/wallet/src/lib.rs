use diesel::{Queryable, Selectable};
use lemmy_db_schema::source::local_user::LocalUser;
use lemmy_db_schema::source::wallet::Wallet;
use lemmy_db_schema::source::wallet_topup::WalletTopup;
use serde::{Deserialize, Serialize};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A wallet view.
pub struct WalletView {
  pub wallet: Wallet,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// A chat wallet topup view, including wallet_topup and local_user.
pub struct WalletTopupView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub wallet_topup: WalletTopup,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_user: LocalUser,
}
