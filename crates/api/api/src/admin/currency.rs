use actix_web::web::{Data, Json};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::source::currency::{Currency, CurrencyInsertForm, CurrencyUpdateForm};
use app_108jobs_db_schema::source::currency_rate_history::{CurrencyRateHistory, CurrencyRateHistoryInsertForm};
use app_108jobs_db_schema::source::pricing_config::{PricingConfig, PricingConfigInsertForm, PricingConfigUpdateForm};
use app_108jobs_db_schema::traits::Crud;
use app_108jobs_db_views_currency::api::{
  CreateCurrency, CurrencyListResponse, CurrencyResponse, CreatePricingConfig,
  GetCurrency, ListPricingConfigs, PricingConfigListResponse,
  PricingConfigResponse, UpdateCurrency, UpdatePricingConfig, GetPricingConfig,
};
use app_108jobs_db_views_currency::{CurrencyView, PricingConfigView};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_utils::error::FastJobResult;
use chrono::Utc;

// ============================================================================
// Currency Admin Endpoints
// ============================================================================

pub async fn admin_list_currencies(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CurrencyListResponse>> {
  is_admin(&local_user_view)?;

  let currencies = Currency::list_all(&mut context.pool()).await?;

  let views = currencies.into_iter().map(|c| CurrencyView { currency: c }).collect();

  Ok(Json(CurrencyListResponse { currencies: views }))
}

pub async fn admin_get_currency(
  data: Json<GetCurrency>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CurrencyResponse>> {
  is_admin(&local_user_view)?;

  let currency = Currency::read(&mut context.pool(), data.id).await?;

  Ok(Json(CurrencyResponse {
    currency: CurrencyView { currency },
  }))
}

pub async fn admin_create_currency(
  data: Json<CreateCurrency>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CurrencyResponse>> {
  is_admin(&local_user_view)?;

  // Check if currency code already exists
  if let Some(_) = Currency::get_by_code(&mut context.pool(), &data.code).await? {
    return Err(app_108jobs_utils::error::FastJobErrorType::CurrencyCodeAlreadyExists.into());
  }

  let form = CurrencyInsertForm {
    code: data.code.clone(),
    name: data.name.clone(),
    symbol: data.symbol.clone(),
    coin_to_currency_rate: data.coin_to_currency_rate,
    decimal_places: data.decimal_places,
    thousands_separator: data.thousands_separator.clone(),
    decimal_separator: data.decimal_separator.clone(),
    symbol_position: data.symbol_position.clone(),
    is_active: true,
    is_default: data.is_default,
    rate_last_updated_by: Some(local_user_view.local_user.id),
  };

  let currency = Currency::create(&mut context.pool(), &form).await?;

  Ok(Json(CurrencyResponse {
    currency: CurrencyView { currency },
  }))
}

pub async fn admin_update_currency(
  data: Json<UpdateCurrency>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CurrencyResponse>> {
  is_admin(&local_user_view)?;

  let existing = Currency::read(&mut context.pool(), data.currency_id).await?;

  // If rate is being updated, record the history
  if let Some(new_rate) = data.coin_to_currency_rate {
    if new_rate != existing.coin_to_currency_rate {
      let rate_history_form = CurrencyRateHistoryInsertForm {
        currency_id: data.currency_id,
        old_rate: existing.coin_to_currency_rate,
        new_rate,
        changed_by: Some(local_user_view.local_user.id),
        reason: data.reason.clone(),
      };
      let _ = CurrencyRateHistory::create(&mut context.pool(), &rate_history_form).await?;
    }
  }

  let form = CurrencyUpdateForm {
    name: data.name.clone(),
    symbol: data.symbol.clone(),
    coin_to_currency_rate: data.coin_to_currency_rate,
    decimal_places: data.decimal_places,
    thousands_separator: data.thousands_separator.clone(),
    decimal_separator: data.decimal_separator.clone(),
    symbol_position: data.symbol_position.clone(),
    is_active: data.is_active,
    is_default: data.is_default,
    rate_last_updated_at: Some(Some(Utc::now())),
    rate_last_updated_by: Some(Some(local_user_view.local_user.id)),
    updated_at: Some(Some(Utc::now())),
  };

  let currency = Currency::update(&mut context.pool(), data.currency_id, &form).await?;

  Ok(Json(CurrencyResponse {
    currency: CurrencyView { currency },
  }))
}

// ============================================================================
// Pricing Config Admin Endpoints
// ============================================================================

pub async fn admin_list_pricing_configs(
  query: Json<ListPricingConfigs>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PricingConfigListResponse>> {
  is_admin(&local_user_view)?;

  let configs = if let Some(currency_id) = query.currency_id {
    PricingConfig::list_by_currency(&mut context.pool(), currency_id).await?
  } else {
    PricingConfig::list_all(&mut context.pool()).await?
  };

  // Build views with currency info
  let mut views = Vec::new();
  for config in configs {
    let currency = Currency::read(&mut context.pool(), config.currency_id).await?;
    views.push(PricingConfigView {
      pricing_config: config,
      currency: currency.into(),
    });
  }

  Ok(Json(PricingConfigListResponse {
    pricing_configs: views,
  }))
}

pub async fn admin_get_pricing_config(
  data: Json<GetPricingConfig>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PricingConfigResponse>> {
  is_admin(&local_user_view)?;

  let config = PricingConfig::read(&mut context.pool(), data.id).await?;
  let currency = Currency::read(&mut context.pool(), config.currency_id).await?;

  Ok(Json(PricingConfigResponse {
    pricing_config: PricingConfigView {
      pricing_config: config,
      currency: currency.into(),
    },
  }))
}

pub async fn admin_create_pricing_config(
  data: Json<CreatePricingConfig>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PricingConfigResponse>> {
  is_admin(&local_user_view)?;

  // Verify currency exists
  let _currency = Currency::read(&mut context.pool(), data.currency_id).await?;

  let form = PricingConfigInsertForm {
    currency_id: data.currency_id,
    name: data.name.clone(),
    base_fare_coin: data.base_fare_coin,
    time_charge_per_minute_coin: data.time_charge_per_minute_coin,
    minimum_charge_minutes: data.minimum_charge_minutes,
    distance_charge_per_km_coin: data.distance_charge_per_km_coin,
    accepts_cash: data.accepts_cash,
    accepts_coin: data.accepts_coin,
    is_active: data.is_active,
  };

  let config = PricingConfig::create(&mut context.pool(), &form).await?;
  let currency = Currency::read(&mut context.pool(), config.currency_id).await?;

  Ok(Json(PricingConfigResponse {
    pricing_config: PricingConfigView {
      pricing_config: config,
      currency: currency.into(),
    },
  }))
}

pub async fn admin_update_pricing_config(
  data: Json<UpdatePricingConfig>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PricingConfigResponse>> {
  is_admin(&local_user_view)?;

  let form = PricingConfigUpdateForm {
    name: data.name.clone(),
    base_fare_coin: data.base_fare_coin,
    time_charge_per_minute_coin: data.time_charge_per_minute_coin,
    minimum_charge_minutes: data.minimum_charge_minutes,
    distance_charge_per_km_coin: data.distance_charge_per_km_coin,
    accepts_cash: data.accepts_cash,
    accepts_coin: data.accepts_coin,
    is_active: data.is_active,
    updated_at: Some(Some(Utc::now())),
  };

  let config = PricingConfig::update(&mut context.pool(), data.config_id, &form).await?;
  let currency = Currency::read(&mut context.pool(), config.currency_id).await?;

  Ok(Json(PricingConfigResponse {
    pricing_config: PricingConfigView {
      pricing_config: config,
      currency: currency.into(),
    },
  }))
}

// ============================================================================
// Helper Functions
// ============================================================================

fn is_admin(local_user_view: &LocalUserView) -> FastJobResult<()> {
  app_108jobs_api_utils::utils::is_admin(local_user_view)
}
