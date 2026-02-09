use crate::newtypes::{CurrencyId, PricingConfigId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use app_108jobs_db_schema_file::schema::pricing_config;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable)
)]
#[cfg_attr(feature = "full", diesel(table_name = pricing_config))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct PricingConfig {
  pub id: PricingConfigId,
  pub currency_id: CurrencyId,
  pub name: String,

  // Stored in Coins internally
  pub base_fare_coin: i32,            // e.g., 5000 Coins
  pub time_charge_per_minute_coin: i32,  // e.g., 100 Coins/minute
  pub minimum_charge_minutes: i32,    // e.g., 10 (charge every 10 min)
  pub distance_charge_per_km_coin: i32,   // e.g., 1000 Coins/km

  pub accepts_cash: bool,
  pub accepts_coin: bool,
  pub is_active: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(
  feature = "full",
  derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
#[cfg_attr(feature = "full", diesel(table_name = pricing_config))]
pub struct PricingConfigInsertForm {
  pub currency_id: CurrencyId,
  pub name: String,
  pub base_fare_coin: i32,
  pub time_charge_per_minute_coin: i32,
  pub minimum_charge_minutes: i32,
  pub distance_charge_per_km_coin: i32,
  pub accepts_cash: bool,
  pub accepts_coin: bool,
  pub is_active: bool,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset, Serialize, Deserialize))]
#[cfg_attr(feature = "full", diesel(table_name = pricing_config))]
pub struct PricingConfigUpdateForm {
  pub name: Option<String>,
  pub base_fare_coin: Option<i32>,
  pub time_charge_per_minute_coin: Option<i32>,
  pub minimum_charge_minutes: Option<i32>,
  pub distance_charge_per_km_coin: Option<i32>,
  pub accepts_cash: Option<bool>,
  pub accepts_coin: Option<bool>,
  pub is_active: Option<bool>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
}
