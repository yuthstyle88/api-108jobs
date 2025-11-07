use crate::WalletTopupView;
use lemmy_db_schema::newtypes::{Coin, LocalUserId, PaginationCursor, WalletId};
use lemmy_db_schema_file::enums::TopupStatus;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

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
  pub target_user_id: LocalUserId,
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
  pub target_user_id: LocalUserId,
  pub qr_id: String,
  pub amount: Coin,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Admin withdraw from user wallet.
pub struct AdminWithdrawWallet {
  pub target_user_id: LocalUserId,
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

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Fetches a list of wallet topups for a user.
pub struct ListWalletTopupsQuery {
  pub amount_min: Option<f64>,
  pub amount_max: Option<f64>,
  /// Optional filter by status (Pending, Success)
  pub status: Option<TopupStatus>,
  /// Optional filter by year of created_at
  pub year: Option<i32>,
  /// Optional filter by month of created_at
  pub month: Option<i32>,
  /// Optional filter by day of created_at
  pub day: Option<i32>,
  /// Pagination cursor for forward/backward navigation
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  /// Limit results (default 20)
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListWalletTopupsResponse {
  pub wallet_topups: Vec<WalletTopupView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}
