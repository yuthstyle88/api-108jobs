use crate::newtypes::{Coin, LocalUserId, WalletId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
pub use lemmy_db_schema_file::enums::TxKind;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::wallet;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::wallet_transaction;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = wallet))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A user wallet for managing balance and transactions.
#[serde(rename_all = "camelCase")]
pub struct Wallet {
  pub id: WalletId,
  pub balance_total: Coin,
  pub balance_available: Coin,
  pub balance_outstanding: Coin,
  pub is_platform: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}
#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A user wallet for managing balance and transactions.
#[serde(rename_all = "camelCase")]
pub struct WalletModel {
  pub platform_wallet_id: WalletId,
  pub balance_total: Coin,
  pub balance_available: Coin,
  pub balance_outstanding: Coin,
}


#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = wallet))]
pub struct WalletInsertForm {
  #[new(default)]
  pub balance_total: Option<Coin>,
  #[new(default)]
  pub balance_available: Option<Coin>,
  #[new(default)]
  pub balance_outstanding: Option<Coin>,
  #[new(default)]
  pub is_platform: Option<bool>,
  #[new(default)]
  pub created_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = wallet))]
pub struct WalletUpdateForm {
  pub balance_total: Option<Coin>,
  pub balance_available: Option<Coin>,
  pub balance_outstanding: Option<Coin>,
  pub is_platform: Option<bool>,
  pub updated_at: Option<DateTime<Utc>>,
}

// crates/db_schema/src/source/wallet_transaction.rs
#[derive(Debug, Clone, Queryable, Identifiable)]
#[diesel(table_name = wallet_transaction)]
pub struct WalletTransaction {
  pub id: i32,
  pub wallet_id: WalletId,
  pub reference_type: String,
  pub reference_id: i32,
  pub kind: TxKind,
  pub amount: Coin,
  pub description: String,
  pub counter_user_id: Option<LocalUserId>,
  pub idempotency_key: String,
}
#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = wallet_transaction))]
pub struct WalletTransactionInsertForm {
  pub wallet_id: WalletId,
  pub reference_type: String,
  pub reference_id: i32,
  pub kind: TxKind,
  pub amount: Coin,
  pub description: String,
  pub counter_user_id: Option<LocalUserId>,
  pub idempotency_key: String,
}

// ฟอร์มสำหรับ DB เท่านั้น (Insertable) — kind เป็น String
#[derive(Debug, Clone)]
pub struct WalletTransactionForm {
  pub wallet_id: WalletId,
  pub reference_type: String,
  pub reference_id: i32,
  pub kind: TxKind,
  pub amount: Coin,
  pub description: String,
  pub counter_user_id: Option<LocalUserId>,
  pub idempotency_key: String,
}

impl From<&WalletTransactionForm> for WalletTransactionInsertForm {
  fn from(f: &WalletTransactionForm) -> Self {
    Self {
      wallet_id: f.wallet_id,
      reference_type: f.reference_type.clone(),
      reference_id: f.reference_id,
      kind: f.kind,
      amount: f.amount, 
      description: f.description.clone(),
      counter_user_id: f.counter_user_id,
      idempotency_key: f.idempotency_key.clone(),
    }
  }
}
#[derive(Debug, AsChangeset, Default, Clone)]
#[diesel(table_name = wallet_transaction)]
pub struct WalletTransactionUpdateForm {
  pub description: Option<String>,
}