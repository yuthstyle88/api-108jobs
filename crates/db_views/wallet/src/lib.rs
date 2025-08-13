use lemmy_db_schema::source::wallet::{Wallet};
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