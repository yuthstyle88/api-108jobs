pub mod api;
#[cfg(feature = "full")]
pub mod impls;
pub mod validator;

use diesel::{Queryable, Selectable};
use app_108jobs_db_schema::source::{bank::Bank, user_bank_account::BankAccount};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
/// A user bank account view with bank information.
pub struct BankAccountView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub user_bank_account: BankAccount,

  #[cfg_attr(feature = "full", diesel(embed))]
  pub bank: Bank,
}
