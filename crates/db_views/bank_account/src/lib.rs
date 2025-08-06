pub mod api;
pub mod impls;

pub use api::*;
use lemmy_db_schema::source::{bank::Bank, user_bank_account::UserBankAccount};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// A user bank account view with bank information.
pub struct UserBankAccountView {
  pub user_bank_account: UserBankAccount,
  pub bank: Bank,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// A bank view.
pub struct BankView {
  pub bank: Bank,
}