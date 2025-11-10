use crate::BankAccountView;
use lemmy_db_schema::newtypes::{BankAccountId, BankId, LocalUserId};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use lemmy_db_schema::source::user_bank_account::BankAccount;

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
/// Set default bank account.
pub struct SetDefaultBankAccount {
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
/// Create a new bank account for user.
pub struct BankAccountForm {
  pub bank_id: BankId,
  pub account_number: String,
  pub account_name: String,
  pub is_default: Option<bool>,
  pub country_id: String,
  pub verification_image: Option<String>, // Base64 encoded image or image path
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Update an existing bank account for user.
pub struct UpdateBankAccount {
  pub bank_account_id: BankAccountId,
  pub bank_id: Option<BankId>,
  pub account_number: Option<String>,
  pub account_name: Option<String>,
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
  pub bank_account: BankAccount,
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
pub struct ListBankAccountsResponse {
  pub bank_accounts: Vec<BankAccountView>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Fetches a list of taglines.
#[serde(rename_all = "camelCase")]
pub struct ListBankAccounts {
  pub bank_accounts: Vec<BankAccountView>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Fetches a random category
pub struct GetBankAccounts {
  pub local_user_id: Option<LocalUserId>,
  pub is_verified: Option<bool>,
}