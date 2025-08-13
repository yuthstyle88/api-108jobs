use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use lemmy_db_schema::newtypes::{BillingId, Coin, CommentId, LocalUserId, PostId, WalletId};

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
/// Create invoice/quotation for job (freelancer creates detailed proposal).
pub struct CreateInvoiceForm {
  pub employer_id: LocalUserId,
  pub post_id: PostId,
  pub comment_id: Option<CommentId>,
  pub amount: Coin,
  pub proposal: String,
  pub project_name: String,
  pub project_details: String,
  pub work_steps: Vec<String>,
  pub working_days: i32,
  pub deliverables: Vec<String>,
  pub note: Option<String>,
  pub starting_day: NaiveDate,  // ISO date string (YYYY-MM-DD)
  pub delivery_day: NaiveDate,  // ISO date string (YYYY-MM-DD)
}

/// Strongly-typed validated wrapper for CreateInvoice
#[derive(Debug, Clone)]
pub struct ValidCreateInvoice(pub CreateInvoiceForm);

impl TryFrom<CreateInvoiceForm> for ValidCreateInvoice {
  type Error = String;
  fn try_from(value: CreateInvoiceForm) -> Result<Self, Self::Error> {
    if value.amount <= 0 {
      return Err("Price must be positive".to_string());
    }
    Ok(ValidCreateInvoice(value))
  }
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
  pub balance: Coin,
  pub transaction_amount: Coin,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Response for creating an invoice.
pub struct CreateInvoiceResponse {
  pub billing_id: BillingId,
  pub issuer_id: LocalUserId,
  pub recipient_id: LocalUserId,
  pub post_id: PostId,
  pub amount: Coin,
  pub status: String,
  pub delivery_timeframe_days: i32,
  pub created_at: String,
  pub success: bool,
}