use crate::newtypes::{CurrencyId, CurrencyRateHistoryId, LocalUserId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use app_108jobs_db_schema_file::schema::currency_rate_history;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable)
)]
#[cfg_attr(feature = "full", diesel(table_name = currency_rate_history))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRateHistory {
  pub id: CurrencyRateHistoryId,
  pub currency_id: CurrencyId,
  pub old_rate: i32,
  pub new_rate: i32,
  pub changed_by: Option<LocalUserId>,
  pub changed_at: DateTime<Utc>,
  pub reason: Option<String>,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(
  feature = "full",
  derive(Insertable)
)]
#[cfg_attr(feature = "full", diesel(table_name = currency_rate_history))]
pub struct CurrencyRateHistoryInsertForm {
  pub currency_id: CurrencyId,
  pub old_rate: i32,
  pub new_rate: i32,
  pub changed_by: Option<LocalUserId>,
  pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRateChange {
  pub old_rate: i32,
  pub new_rate: i32,
  pub change_percent: f64,
  pub changed_at: DateTime<Utc>,
  pub changed_by: Option<LocalUserId>,
  pub reason: Option<String>,
}

impl CurrencyRateHistory {
  /// Calculate the percentage change from old to new rate
  pub fn change_percent(&self) -> f64 {
    if self.old_rate == 0 {
      100.0 // Treat as 100% increase if old was 0
    } else {
      ((self.new_rate - self.old_rate) as f64 / self.old_rate as f64) * 100.0
    }
  }
}
