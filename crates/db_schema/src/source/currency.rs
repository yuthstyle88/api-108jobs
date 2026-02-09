use crate::newtypes::{CurrencyId, LocalUserId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use app_108jobs_db_schema_file::schema::currency;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable)
)]
#[cfg_attr(feature = "full", diesel(table_name = currency))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct Currency {
  pub id: CurrencyId,
  pub code: String,
  pub name: String,
  pub symbol: String,

  // Conversion rate: how many currency units = 1 Coin
  // THB: 1 (1 Coin = 0.01 THB, so 100 Coins = 1 THB)
  // IDR: 100 (1 Coin = 100 Rupiah)
  // VND: 100 (1 Coin = 100 Dong)
  pub coin_to_currency_rate: i32,

  // Display formatting
  pub decimal_places: i32,
  pub thousands_separator: String,
  pub decimal_separator: String,
  pub symbol_position: String,

  pub is_active: bool,
  pub is_default: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  pub rate_last_updated_at: Option<DateTime<Utc>>,
  pub rate_last_updated_by: Option<LocalUserId>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(
  feature = "full",
  derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
#[cfg_attr(feature = "full", diesel(table_name = currency))]
pub struct CurrencyInsertForm {
  pub code: String,
  pub name: String,
  pub symbol: String,
  pub coin_to_currency_rate: i32,
  pub decimal_places: i32,
  pub thousands_separator: String,
  pub decimal_separator: String,
  pub symbol_position: String,
  pub is_active: bool,
  pub is_default: bool,
  pub rate_last_updated_by: Option<LocalUserId>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset, Serialize, Deserialize))]
#[cfg_attr(feature = "full", diesel(table_name = currency))]
pub struct CurrencyUpdateForm {
  pub name: Option<String>,
  pub symbol: Option<String>,
  pub coin_to_currency_rate: Option<i32>,
  pub decimal_places: Option<i32>,
  pub thousands_separator: Option<String>,
  pub decimal_separator: Option<String>,
  pub symbol_position: Option<String>,
  pub is_active: Option<bool>,
  pub is_default: Option<bool>,
  pub rate_last_updated_at: Option<Option<DateTime<Utc>>>,
  pub rate_last_updated_by: Option<Option<LocalUserId>>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct CurrencyInfo {
  pub id: CurrencyId,
  pub code: String,
  pub name: String,
  pub symbol: String,
  pub coin_to_currency_rate: i32,
  pub decimal_places: i32,
  pub thousands_separator: String,
  pub decimal_separator: String,
  pub symbol_position: String,
  pub is_default: bool,
}

impl From<Currency> for CurrencyInfo {
  fn from(c: Currency) -> Self {
    Self {
      id: c.id,
      code: c.code,
      name: c.name,
      symbol: c.symbol,
      coin_to_currency_rate: c.coin_to_currency_rate,
      decimal_places: c.decimal_places,
      thousands_separator: c.thousands_separator,
      decimal_separator: c.decimal_separator,
      symbol_position: c.symbol_position,
      is_default: c.is_default,
    }
  }
}

impl Currency {
  /// Convert Coins to local currency value using this currency's rate
  pub fn coins_to_currency(&self, coins: i32) -> f64 {
    coins as f64 * self.coin_to_currency_rate as f64
  }

  /// Convert local currency value to Coins
  pub fn currency_to_coins(&self, amount: f64) -> i32 {
    (amount / self.coin_to_currency_rate as f64) as i32
  }

  /// Format Coins as a display string in this currency
  /// Example: 5000 Coins with THB → "฿50.00"
  ///          5000 Coins with IDR → "Rp500.000"
  pub fn format_coins(&self, coins: i32) -> String {
    let value = self.coins_to_currency(coins);
    self.format_value(value)
  }

  /// Format a local currency value as a display string
  pub fn format_value(&self, value: f64) -> String {
    let abs_value = value.abs();

    // Format with appropriate decimal places
    let formatted_number: String = if self.decimal_places == 0 {
      format!("{:.0}", abs_value)
    } else {
      format!("{:.1$}", abs_value, self.decimal_places as usize)
    };

    // Split integer and decimal parts
    let (integer_part, decimal_part) = if let Some(pos) = formatted_number.find('.') {
      (&formatted_number[..pos], &formatted_number[pos + 1..])
    } else {
      (&formatted_number[..], "")
    };

    // Add thousands separator to integer part
    let with_separator = self.add_thousands_separator(integer_part);

    // Combine with decimal part
    let with_decimal = if decimal_part.is_empty() {
      with_separator.clone()
    } else {
      format!("{}{}{}", with_separator, self.decimal_separator, decimal_part)
    };

    // Add negative sign if needed
    let signed = if value < 0.0 {
      format!("-{}", with_decimal)
    } else {
      with_decimal
    };

    // Add symbol
    match self.symbol_position.as_str() {
      "suffix" => format!("{} {}", signed, self.symbol),
      _ => format!("{}{}", self.symbol, signed),
    }
  }

  fn add_thousands_separator(&self, s: &str) -> String {
    s.chars()
      .rev()
      .collect::<Vec<char>>()
      .chunks(3)
      .map(|chunk| chunk.iter().collect::<String>())
      .collect::<Vec<String>>()
      .join(&self.thousands_separator)
      .chars()
      .rev()
      .collect::<String>()
  }
}
