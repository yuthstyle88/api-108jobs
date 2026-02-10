use crate::{TopUpRequestView, WithdrawRequestView};
use app_108jobs_db_schema::newtypes::{
  BankAccountId, Coin, CurrencyId, LocalUserId, PaginationCursor, WalletId, WithdrawRequestId,
};
use app_108jobs_db_schema_file::enums::{TopUpStatus, WithdrawStatus};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Get wallet for a user.
pub struct GetWallet {
  pub user_id: Option<i32>,
}

#[derive(Debug, Clone)]
/// Update wallet balance.
pub struct UpdateWallet {
  pub amount: Coin,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
/// Update wallet balance.
pub struct UpdateWalletRequest {
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
/// Note: amount is no longer needed here - it's fetched from the TopUpRequest
pub struct AdminTopUpWallet {
  pub target_user_id: LocalUserId,
  pub qr_id: String,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Admin withdraw from user wallet.
pub struct AdminWithdrawWallet {
  pub target_user_id: LocalUserId,
  pub withdrawal_id: WithdrawRequestId,
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
pub struct ListTopUpRequestQuery {
  pub amount_min: Option<f64>,
  pub amount_max: Option<f64>,
  /// Optional filter by status (Pending, Success)
  pub status: Option<TopUpStatus>,
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
pub struct ListTopUpRequestResponse {
  pub top_up_requests: Vec<TopUpRequestView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Client submits a withdrawal request.
pub struct SubmitWithdrawRequest {
  pub wallet_id: WalletId,
  pub bank_account_id: BankAccountId,
  pub amount: Coin,
  pub currency_id: CurrencyId,
  pub reason: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Fetches a list of withdrawal requests for a user.
pub struct ListWithdrawRequestQuery {
  /// Minimum withdrawal amount filter
  pub amount_min: Option<Coin>,
  /// Maximum withdrawal amount filter
  pub amount_max: Option<Coin>,
  /// Optional filter by status (Pending, Rejected, Completed)
  pub status: Option<WithdrawStatus>,
  /// Optional filter by year of created_at
  pub year: Option<i32>,
  /// Optional filter by month of created_at
  pub month: Option<i32>,
  /// Optional filter by day of created_at
  pub day: Option<i32>,
  /// Pagination cursor for forward/backward navigation
  pub page_cursor: Option<PaginationCursor>,
  /// If true, fetch results before the cursor instead of after
  pub page_back: Option<bool>,
  /// Limit results (default 20)
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
/// Response containing a list of withdrawal requests and pagination info.
pub struct ListWithdrawRequestResponse {
  /// The list of withdrawal requests (with user/bank info if needed)
  pub withdraw_requests: Vec<WithdrawRequestView>,
  /// The pagination cursor to fetch the next page
  pub next_page: Option<PaginationCursor>,
  /// The pagination cursor to fetch the previous page
  pub prev_page: Option<PaginationCursor>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Admin reject withdrawal request from user wallet.
pub struct RejectWithdrawalRequest {
  pub withdrawal_id: WithdrawRequestId,
  pub reason: String,
}
