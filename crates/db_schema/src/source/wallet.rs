use crate::newtypes::{WalletId, PersonId};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
// Schema import handled by diesel(table_name) attribute
#[cfg(feature = "full")]
use diesel::prelude::*;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = lemmy_db_schema_file::schema::wallet))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Enhanced user wallet with comprehensive balance tracking and financial controls.
#[serde(rename_all = "camelCase")]
pub struct Wallet {
  pub id: WalletId,
  /// Available balance for spending
  pub available_balance: BigDecimal,
  /// Money held in escrow for ongoing jobs  
  pub escrow_balance: BigDecimal,
  /// Incoming pending transactions
  pub pending_in: BigDecimal,
  /// Outgoing pending transactions
  pub pending_out: BigDecimal,
  /// Reserved balance for platform fees/holds
  pub reserved_balance: BigDecimal,
  /// Account frozen status
  pub is_frozen: bool,
  /// Reason for account freeze
  pub freeze_reason: Option<String>,
  /// Currency code (THB, VND, USD, etc.)
  pub currency: String,
  /// Version for optimistic locking
  pub version: i32,
  /// Timestamps
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  /// Last transaction timestamp
  pub last_transaction_at: Option<DateTime<Utc>>,
  /// Direct link to person (federation support)
  pub person_id: PersonId,
  /// Who made the last update
  pub updated_by_person_id: Option<PersonId>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = lemmy_db_schema_file::schema::wallet))]
pub struct WalletInsertForm {
  #[new(default)]
  pub available_balance: Option<BigDecimal>,
  #[new(default)]
  pub escrow_balance: Option<BigDecimal>,
  #[new(default)]
  pub pending_in: Option<BigDecimal>,
  #[new(default)]
  pub pending_out: Option<BigDecimal>,
  #[new(default)]
  pub reserved_balance: Option<BigDecimal>,
  #[new(default)]
  pub is_frozen: Option<bool>,
  #[new(default)]
  pub freeze_reason: Option<String>,
  #[new(default)]
  pub currency: Option<String>,
  #[new(default)]
  pub version: Option<i32>,
  #[new(default)]
  pub created_at: Option<DateTime<Utc>>,
  pub person_id: PersonId,
  #[new(default)]
  pub updated_by_person_id: Option<PersonId>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = lemmy_db_schema_file::schema::wallet))]
pub struct WalletUpdateForm {
  pub available_balance: Option<BigDecimal>,
  pub escrow_balance: Option<BigDecimal>,
  pub pending_in: Option<BigDecimal>,
  pub pending_out: Option<BigDecimal>,
  pub reserved_balance: Option<BigDecimal>,
  pub is_frozen: Option<bool>,
  pub freeze_reason: Option<String>,
  pub currency: Option<String>,
  // Note: version is handled by trigger
  pub updated_at: Option<DateTime<Utc>>,
  pub last_transaction_at: Option<DateTime<Utc>>,
  pub updated_by_person_id: Option<PersonId>,
}