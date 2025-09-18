pub mod api;
#[cfg(feature = "full")]
pub mod impls;

use lemmy_db_schema::source::{bank::Bank, user_bank_account::BankAccount};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
/// A user bank account view with bank information.
pub struct BankAccountView {
  pub user_bank_account: BankAccount,
  pub bank: Bank,
}

