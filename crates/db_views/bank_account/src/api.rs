use crate::BankAccountView;
use lemmy_db_schema::newtypes::{BankAccountId, BankId, LocalUserId, PaginationCursor};
use lemmy_utils::error::{FastJobError, FastJobErrorType};
use lemmy_utils::utils::validation::validate_bank_account;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

impl TryFrom<BankAccountForm> for CreateBankAccount {
  type Error = FastJobError;

  fn try_from(data: BankAccountForm) -> Result<Self, Self::Error> {
    // Validate account number presence and format by country
    let acc_num = data.account_number.trim();
    if acc_num.is_empty() || !validate_bank_account(&data.country_id, acc_num) {
      return Err(FastJobErrorType::InvalidField("Invalid account number".to_string()).into());
    }

    // Validate account name
    if data.account_name.trim().is_empty() {
      return Err(FastJobErrorType::InvalidField("Invalid account name".to_string()).into());
    }

    Ok(CreateBankAccount {
      bank_id: data.bank_id,
      account_number: data.account_number,
      account_name: data.account_name,
      is_default: None,
      verification_image: None,
    })
  }
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
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
  pub bank_account: BankAccountView,
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// List unverified bank accounts (admin only).
pub struct ListBankAccountsResponse {
  pub bank_accounts: Vec<BankAccountView>,
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
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

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct ListBankAccountQuery {
  pub limit: Option<i64>,
  pub is_verified: Option<bool>,
  pub is_default: Option<bool>,
  pub year: Option<i32>,
  pub month: Option<i32>,
  pub day: Option<i32>,
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
}