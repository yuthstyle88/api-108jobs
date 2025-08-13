use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::coin;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use crate::newtypes::Coin;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = coin))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Platform coin metadata (single row when using one coin).
#[serde(rename_all = "camelCase")]
pub struct CoinModel {
  pub id: i32,
  /// Ticker/symbol, e.g. "FJC" (FastJob Coin)
  pub code: String,
  /// Human-readable name
  pub name: String,
  pub supply_total: Coin,
  /// Circulating supply (mint - burn). Derived from wallet transactions; keep in sync via reconciliation.
  pub supply_minted_total: Coin,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = coin))]
pub struct CoinModelInsertForm {
  pub code: String,
  pub name: String,
  pub supply_total: Option<Coin>,
  pub supply_minted_total: Option<Coin>,
  #[new(default)]
  pub created_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = coin))]
pub struct CoinModelUpdateForm {
  pub code: Option<String>,
  pub name: Option<String>,
  pub supply_total: Option<Coin>,
  pub supply_minted_total: Option<Coin>,
  pub updated_at: Option<DateTime<Utc>>,
}
