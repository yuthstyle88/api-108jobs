use crate::newtypes::{UserBankAccountId, LocalUserId, BankId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::user_bank_accounts;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = user_bank_accounts))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct UserBankAccount {
  pub id: UserBankAccountId,
  pub user_id: LocalUserId,
  pub bank_id: BankId,
  pub account_number: String,
  pub account_name: String,
  pub is_default: Option<bool>,
  pub is_verified: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = user_bank_accounts))]
pub struct UserBankAccountInsertForm {
  pub user_id: LocalUserId,
  pub bank_id: BankId,
  pub account_number: String,
  pub account_name: String,
  pub is_default: Option<bool>,
  pub verification_image_path: Option<String>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = user_bank_accounts))]
pub struct UserBankAccountUpdateForm {
  pub bank_id: Option<BankId>,
  pub account_number: Option<String>,
  pub account_name: Option<String>,
  pub is_default: Option<bool>,
  pub is_verified: Option<bool>,
  pub updated_at: Option<DateTime<Utc>>,
  pub verification_image_path: Option<String>,
}