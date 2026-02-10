use app_108jobs_db_schema::source::currency::{Currency, CurrencyInfo};
use app_108jobs_db_schema::source::currency_rate_history::CurrencyRateHistory;
use app_108jobs_db_schema::source::pricing_config::PricingConfig;
use app_108jobs_db_schema::source::local_user::LocalUser;
use serde::{Deserialize, Serialize};

pub mod api;
pub mod validator;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A currency view.
pub struct CurrencyView {
  pub currency: Currency,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A pricing config view with currency info.
pub struct PricingConfigView {
  pub pricing_config: PricingConfig,
  pub currency: CurrencyInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(diesel::Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// A currency rate history view with admin info.
pub struct CurrencyRateHistoryView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub rate_history: CurrencyRateHistory,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub currency: CurrencyInfo,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub changed_by_user: Option<LocalUser>,
}
