use lemmy_db_schema::newtypes::{BankId, BankAccountId, LocalUserId};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use crate::BankAccountView;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// A comment response.
pub struct BankAccountResponse {
  pub bank_account_view: BankAccountView,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Get list of available banks.
pub struct GetBanks {
  pub country: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Get user bank accounts.
pub struct GetUserBankAccounts {}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Create user bank account.
pub struct CreateUserBankAccount {
  pub bank_id: BankId,
  pub account_number: String,
  pub account_name: String,
  pub is_default: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Set default bank account.
pub struct SetDefaultBankAccount {
  pub bank_account_id: BankAccountId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Delete bank account.
pub struct DeleteUserBankAccount {
  pub bank_account_id: BankAccountId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Create a new bank account for user.
pub struct CreateBankAccount {
  pub bank_id: BankId,
  pub account_number: String,
  pub account_name: String,
  pub is_default: Option<bool>,
  pub verification_image: Option<String>, // Base64 encoded image or image path
}


#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Delete bank account.
pub struct DeleteBankAccount {
  pub bank_account_id: BankAccountId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Bank information.
pub struct BankResponse {
  pub id: BankId,
  pub name: String,
  pub country_id: String,
  pub bank_code: Option<String>,
  pub swift_code: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Bank account operation response.
pub struct BankAccountOperationResponse {
  pub bank_account_id: BankAccountId,
  pub success: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Verify a bank account (admin only).
pub struct VerifyBankAccount {
  pub bank_account_id: BankAccountId,
  pub verified: bool,
  pub admin_notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// List unverified bank accounts (admin only).
pub struct ListUnverifiedBankAccountsResponse {
  pub bank_accounts: Vec<UnverifiedBankAccountResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Unverified bank account response for admin.
pub struct UnverifiedBankAccountResponse {
  pub id: BankAccountId,
  pub user_id: LocalUserId,
  pub bank_id: BankId,
  pub bank_name: String,
  pub bank_country_id: String,
  pub account_number: String,
  pub account_name: String,
  pub is_default: bool,
  pub verification_image_path: Option<String>,
  pub created_at: String,
}