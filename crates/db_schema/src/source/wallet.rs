use crate::newtypes::WalletId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::wallet;

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
  /// Available balance for spending
  pub balance: f64,
  pub escrow_balance: f64,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  /// Money held in escrow for ongoing jobs
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = wallet))]
pub struct WalletInsertForm {
  #[new(default)]
  pub balance: Option<f64>,
  #[new(default)]
  pub escrow_balance: Option<f64>,
  #[new(default)]
  pub created_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = wallet))]
pub struct WalletUpdateForm {
  pub balance: Option<f64>,
  pub escrow_balance: Option<f64>,
  pub updated_at: Option<DateTime<Utc>>,
}