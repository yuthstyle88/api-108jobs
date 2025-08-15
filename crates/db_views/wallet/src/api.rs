use lemmy_db_schema::newtypes::{Coin, WalletId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Get wallet for a user.
pub struct GetWallet {
  pub user_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Update wallet balance.
pub struct UpdateWallet {
  pub amount: Coin,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Response for getting wallet.
pub struct GetWalletResponse {
  pub wallet_id: WalletId,
  pub balance: Coin,
  pub escrow_balance: Coin, // Money held in escrow
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Add funds to wallet (deposit).
pub struct DepositWallet {
  pub amount: Coin,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Response for wallet operations.
pub struct WalletOperationResponse {
  pub wallet_id: WalletId,
  pub balance: Coin,
  pub transaction_amount: Coin,
  pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Admin top up user wallet.
pub struct AdminTopUpWallet {
  pub wallet_id: WalletId,
  pub amount: Coin,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Admin withdraw from user wallet.
pub struct AdminWithdrawWallet {
  pub wallet_id: WalletId,
  pub amount: Coin,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Response for admin wallet operations.
pub struct AdminWalletOperationResponse {
  pub wallet_id: WalletId,
  pub new_balance: Coin,
  pub operation_amount: Coin,
  pub reason: String,
  pub success: bool,
}