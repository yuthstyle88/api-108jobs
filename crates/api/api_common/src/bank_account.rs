use lemmy_db_schema::newtypes::{BankId, UserBankAccountId, LocalUserId};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(ts_rs::TS))]
#[cfg_attr(feature = "full", ts(export))]
#[serde(rename_all = "camelCase")]
/// Create a new bank account for user.
pub struct CreateUserBankAccount {
  pub bank_id: BankId,
  pub account_number: String,
  pub account_name: String,
  pub is_default: Option<bool>,
  pub verification_image: Option<String>, // Base64 encoded image or image path
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(ts_rs::TS))]
#[cfg_attr(feature = "full", ts(export))]
#[serde(rename_all = "camelCase")]
/// Set default bank account.
pub struct SetDefaultBankAccount {
  pub bank_account_id: UserBankAccountId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(ts_rs::TS))]
#[cfg_attr(feature = "full", ts(export))]
#[serde(rename_all = "camelCase")]
/// Delete bank account.
pub struct DeleteBankAccount {
  pub bank_account_id: UserBankAccountId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(ts_rs::TS))]
#[cfg_attr(feature = "full", ts(export))]
#[serde(rename_all = "camelCase")]
/// Bank information.
pub struct BankResponse {
  pub id: BankId,
  pub name: String,
  pub country: String,
  pub bank_code: Option<String>,
  pub swift_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(ts_rs::TS))]
#[cfg_attr(feature = "full", ts(export))]
#[serde(rename_all = "camelCase")]
/// User bank account response.
pub struct UserBankAccountResponse {
  pub id: UserBankAccountId,
  pub bank_id: BankId,
  pub bank_name: String,
  pub bank_country: String,
  pub account_number: String,
  pub account_name: String,
  pub is_default: bool,
  pub is_verified: bool,
  pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(ts_rs::TS))]
#[cfg_attr(feature = "full", ts(export))]
#[serde(rename_all = "camelCase")]
/// List of banks response.
pub struct ListBanksResponse {
  pub banks: Vec<BankResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(ts_rs::TS))]
#[cfg_attr(feature = "full", ts(export))]
#[serde(rename_all = "camelCase")]
/// List of user bank accounts response.
pub struct ListUserBankAccountsResponse {
  pub bank_accounts: Vec<UserBankAccountResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(ts_rs::TS))]
#[cfg_attr(feature = "full", ts(export))]
#[serde(rename_all = "camelCase")]
/// Bank account operation response.
pub struct BankAccountOperationResponse {
  pub bank_account_id: UserBankAccountId,
  pub success: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(ts_rs::TS))]
#[cfg_attr(feature = "full", ts(export))]
#[serde(rename_all = "camelCase")]
/// Verify bank account (admin only).
pub struct VerifyBankAccount {
  pub bank_account_id: UserBankAccountId,
  pub verified: bool,
  pub admin_notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(ts_rs::TS))]
#[cfg_attr(feature = "full", ts(export))]
#[serde(rename_all = "camelCase")]
/// List unverified bank accounts (admin only).
pub struct ListUnverifiedBankAccountsResponse {
  pub bank_accounts: Vec<UnverifiedBankAccountResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(ts_rs::TS))]
#[cfg_attr(feature = "full", ts(export))]
#[serde(rename_all = "camelCase")]
/// Unverified bank account response for admin.
pub struct UnverifiedBankAccountResponse {
  pub id: UserBankAccountId,
  pub user_id: LocalUserId,
  pub bank_id: BankId,
  pub bank_name: String,
  pub bank_country: String,
  pub account_number: String,
  pub account_name: String,
  pub is_default: bool,
  pub verification_image_path: Option<String>,
  pub created_at: String,
}