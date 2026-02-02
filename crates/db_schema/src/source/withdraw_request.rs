use crate::newtypes::{BankAccountId, Coin, LocalUserId, WalletId, WithdrawRequestId};
use chrono::{DateTime, Utc};
use app_108jobs_db_schema_file::enums::WithdrawStatus;

#[cfg(feature = "full")]
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
use {i_love_jesus::CursorKeysModule, app_108jobs_db_schema_file::schema::withdraw_requests};

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = withdraw_requests))]
#[cfg_attr(feature = "full", diesel(primary_key(id)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = withdraw_requests_keys))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[serde(rename_all = "camelCase")]
pub struct WithdrawRequest {
  pub id: WithdrawRequestId,
  pub local_user_id: LocalUserId,
  pub wallet_id: WalletId,
  pub user_bank_account_id: BankAccountId,
  pub amount: Coin,
  pub status: WithdrawStatus,
  pub reason: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(
  feature = "full",
  derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
#[cfg_attr(feature = "full", diesel(table_name = withdraw_requests))]
pub struct WithdrawRequestInsertForm {
  pub local_user_id: LocalUserId,
  pub wallet_id: WalletId,
  pub user_bank_account_id: BankAccountId,
  pub amount: Coin,
  pub reason: Option<String>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(Serialize, Deserialize, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = withdraw_requests))]
pub struct WithdrawRequestUpdateForm {
  pub status: Option<WithdrawStatus>,
  pub updated_at: Option<DateTime<Utc>>,
  pub reason: Option<Option<String>>,
}
