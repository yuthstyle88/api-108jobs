use serde::{Deserialize, Serialize};
use lemmy_db_schema::newtypes::{BillingId, CommentId, LocalUserId, PostId, WalletId};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Get wallet for a user.
pub struct GetWallet {
  pub user_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Update wallet balance.
pub struct UpdateWallet {
  pub amount: f64,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Response for getting wallet.
pub struct GetWalletResponse {
  pub wallet_id: WalletId,
  pub balance: f64,
  pub escrow_balance: f64, // Money held in escrow
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Add funds to wallet (deposit).
pub struct DepositWallet {
  pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Create invoice/quotation for job (freelancer creates detailed proposal).
pub struct CreateInvoice {
  pub employer_id: LocalUserId,
  pub post_id: PostId,
  pub comment_id: Option<CommentId>,
  pub price: f64,
  pub proposal: String,
  pub name: String,
  pub job_description: String,
  pub work_steps: Vec<String>,
  pub revise_times: i32,
  pub revise_description: String,
  pub working_days: i32,
  pub deliverables: Vec<String>,
  pub note: Option<String>,
  pub starting_day: String,  // ISO date string (YYYY-MM-DD)
  pub delivery_day: String,  // ISO date string (YYYY-MM-DD)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Approve quotation and convert to order (employer approves freelancer's quotation).
pub struct ApproveQuotation {
  pub billing_id: BillingId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Pay invoice (employer pays, money goes to escrow).
pub struct PayInvoice {
  pub billing_id: BillingId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Submit completed work (freelancer submits work).
pub struct SubmitWork {
  pub billing_id: BillingId,
  pub work_description: String,
  pub deliverable_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Request revision of submitted work (employer requests changes).
pub struct RequestRevision {
  pub billing_id: BillingId,
  pub revision_feedback: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Update work after revision request (freelancer updates work).
pub struct UpdateWorkAfterRevision {
  pub billing_id: BillingId,
  pub updated_work_description: String,
  pub updated_deliverable_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Approve work and release payment (employer approves).
pub struct ApproveWork {
  pub billing_id: BillingId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Response for wallet operations.
pub struct WalletOperationResponse {
  pub wallet_id: WalletId,
  pub balance: f64,
  pub escrow_balance: f64,
  pub transaction_amount: f64,
  pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Response for billing operations.
pub struct BillingOperationResponse {
  pub billing_id: BillingId,
  pub status: String,
  pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Admin top up user wallet.
pub struct AdminTopUpWallet {
  pub user_id: LocalUserId,
  pub amount: f64,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Admin withdraw from user wallet.
pub struct AdminWithdrawWallet {
  pub user_id: LocalUserId,
  pub amount: f64,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Response for admin wallet operations.
pub struct AdminWalletOperationResponse {
  pub user_id: LocalUserId,
  pub wallet_id: WalletId,
  pub previous_balance: f64,
  pub new_balance: f64,
  pub operation_amount: f64,
  pub reason: String,
  pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Response for creating an invoice.
pub struct CreateInvoiceResponse {
  pub billing_id: BillingId,
  pub freelancer_id: LocalUserId,
  pub employer_id: LocalUserId,
  pub post_id: PostId,
  pub amount: f64,
  pub status: String,
  pub max_revisions: i32,
  pub delivery_timeframe_days: i32,
  pub created_at: String,
  pub success: bool,
}