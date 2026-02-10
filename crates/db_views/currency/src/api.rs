use crate::{CurrencyView, PricingConfigView, CurrencyRateHistoryView};
use app_108jobs_db_schema::newtypes::{CurrencyId, PricingConfigId};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// ============================================================================
// Currency Admin API Types
// ============================================================================

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Create a new currency (admin only).
pub struct CreateCurrencyRequest {
  pub code: String,
  pub name: String,
  pub symbol: String,
  pub numeric_code: i32,
  pub coin_to_currency_rate: i32,
  pub decimal_places: i32,
  pub thousands_separator: String,
  pub decimal_separator: String,
  pub symbol_position: String,
  pub is_default: bool,
}

/// Internal type for currency creation after validation
#[derive(Debug, Clone)]
pub struct CreateCurrency {
  pub code: String,
  pub name: String,
  pub symbol: String,
  pub numeric_code: i32,
  pub coin_to_currency_rate: i32,
  pub decimal_places: i32,
  pub thousands_separator: String,
  pub decimal_separator: String,
  pub symbol_position: String,
  pub is_default: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Update a currency (admin only).
pub struct UpdateCurrencyRequest {
  pub currency_id: CurrencyId,
  pub name: Option<String>,
  pub symbol: Option<String>,
  pub numeric_code: Option<i32>,
  pub coin_to_currency_rate: Option<i32>,
  pub decimal_places: Option<i32>,
  pub thousands_separator: Option<String>,
  pub decimal_separator: Option<String>,
  pub symbol_position: Option<String>,
  pub is_active: Option<bool>,
  pub is_default: Option<bool>,
  pub reason: Option<String>, // Reason for rate change (if rate is being updated)
}

/// Internal type for currency update after validation
#[derive(Debug, Clone)]
pub struct UpdateCurrency {
  pub currency_id: CurrencyId,
  pub name: Option<String>,
  pub symbol: Option<String>,
  pub numeric_code: Option<i32>,
  pub coin_to_currency_rate: Option<i32>,
  pub decimal_places: Option<i32>,
  pub thousands_separator: Option<String>,
  pub decimal_separator: Option<String>,
  pub symbol_position: Option<String>,
  pub is_active: Option<bool>,
  pub is_default: Option<bool>,
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Get currency by ID.
pub struct GetCurrency {
  pub id: CurrencyId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Response for single currency.
pub struct CurrencyResponse {
  pub currency: CurrencyView,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Response for currency list.
pub struct CurrencyListResponse {
  pub currencies: Vec<CurrencyView>,
}

// ============================================================================
// Pricing Config Admin API Types
// ============================================================================

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Create a new pricing config (admin only).
pub struct CreatePricingConfigRequest {
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

/// Internal type for pricing config creation after validation
#[derive(Debug, Clone)]
pub struct CreatePricingConfig {
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

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Update a pricing config (admin only).
pub struct UpdatePricingConfigRequest {
  pub config_id: PricingConfigId,
  pub name: Option<String>,
  pub base_fare_coin: Option<i32>,
  pub time_charge_per_minute_coin: Option<i32>,
  pub minimum_charge_minutes: Option<i32>,
  pub distance_charge_per_km_coin: Option<i32>,
  pub accepts_cash: Option<bool>,
  pub accepts_coin: Option<bool>,
  pub is_active: Option<bool>,
}

/// Internal type for pricing config update after validation
#[derive(Debug, Clone)]
pub struct UpdatePricingConfig {
  pub config_id: PricingConfigId,
  pub name: Option<String>,
  pub base_fare_coin: Option<i32>,
  pub time_charge_per_minute_coin: Option<i32>,
  pub minimum_charge_minutes: Option<i32>,
  pub distance_charge_per_km_coin: Option<i32>,
  pub accepts_cash: Option<bool>,
  pub accepts_coin: Option<bool>,
  pub is_active: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Get pricing config by ID.
pub struct GetPricingConfig {
  pub id: PricingConfigId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// List pricing configs with optional filters.
pub struct ListPricingConfigs {
  pub currency_id: Option<CurrencyId>,
  pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Response for single pricing config.
pub struct PricingConfigResponse {
  pub pricing_config: PricingConfigView,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Response for pricing config list.
pub struct PricingConfigListResponse {
  pub pricing_configs: Vec<PricingConfigView>,
}

// ============================================================================
// Currency Rate History API Types
// ============================================================================

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// List currency rate changes.
pub struct ListCurrencyRateHistory {
  pub currency_id: Option<CurrencyId>,
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Response for currency rate history.
pub struct CurrencyRateHistoryResponse {
  pub rate_changes: Vec<CurrencyRateHistoryView>,
}
