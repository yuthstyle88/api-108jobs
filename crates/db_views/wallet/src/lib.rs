use app_108jobs_db::source::{
  local_user::LocalUser,
  top_up_request::TopUpRequest,
  user_bank_account::BankAccount,
  wallet::Wallet,
  withdraw_request::WithdrawRequest,
};
use diesel::{Queryable, Selectable};
use serde::{Deserialize, Serialize};

pub mod api;
pub mod validator;
pub use api::{ListWithdrawRequestQuery, ListWithdrawRequestResponse, SubmitWithdrawRequest};
pub use validator::ValidSubmitWithdrawRequest;
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
/// A Top-up Request view, including top_up_request and local_user.
pub struct TopUpRequestView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub top_up_request: TopUpRequest,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_user: LocalUser,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// A Withdrawal Request view, including withdraw_request, local_user and bank_account.
pub struct WithdrawRequestView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub withdraw_request: WithdrawRequest,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_user: LocalUser,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub bank_account: BankAccount,
}
