//! Validation logic for currency requests
use crate::api::{
  CreateCurrency, CreateCurrencyRequest, CreatePricingConfig, CreatePricingConfigRequest,
  UpdateCurrency, UpdateCurrencyRequest, UpdatePricingConfig, UpdatePricingConfigRequest,
};
use app_108jobs_db_schema::newtypes::{CurrencyId, PricingConfigId};
use app_108jobs_utils::error::{FastJobError, FastJobErrorType, FastJobResult};

/// Validates that currency code is not empty and is uppercase
pub fn validate_currency_code(code: &str) -> FastJobResult<()> {
  let trimmed = code.trim();
  if trimmed.is_empty() {
    return Err(FastJobErrorType::InvalidField("currency code cannot be empty".to_string()).into());
  }
  if trimmed.len() != 3 || !trimmed.chars().all(|c| c.is_ascii_uppercase()) {
    return Err(FastJobErrorType::InvalidField("currency code must be 3 uppercase letters".to_string()).into());
  }
  Ok(())
}

/// Validates that currency name is not empty
pub fn validate_currency_name(name: &str) -> FastJobResult<()> {
  if name.trim().is_empty() {
    return Err(FastJobErrorType::InvalidField("currency name cannot be empty".to_string()).into());
  }
  Ok(())
}

/// Validates that currency symbol is not empty
pub fn validate_currency_symbol(symbol: &str) -> FastJobResult<()> {
  if symbol.trim().is_empty() {
    return Err(FastJobErrorType::InvalidField("currency symbol cannot be empty".to_string()).into());
  }
  Ok(())
}

/// Validates that numeric code is positive
pub fn validate_numeric_code(code: i32) -> FastJobResult<()> {
  if code <= 0 {
    return Err(FastJobErrorType::InvalidField("numeric code must be positive".to_string()).into());
  }
  Ok(())
}

/// Validates that coin to currency rate is positive
pub fn validate_coin_to_currency_rate(rate: i32) -> FastJobResult<()> {
  if rate <= 0 {
    return Err(FastJobErrorType::InvalidField("coin to currency rate must be positive".to_string()).into());
  }
  Ok(())
}

/// Validates that decimal places is non-negative
pub fn validate_decimal_places(decimal_places: i32) -> FastJobResult<()> {
  if decimal_places < 0 {
    return Err(FastJobErrorType::InvalidField("decimal places must be non-negative".to_string()).into());
  }
  Ok(())
}

/// Validates that currency ID is positive
pub fn validate_currency_id(currency_id: CurrencyId) -> FastJobResult<()> {
  if currency_id.0 <= 0 {
    return Err(FastJobErrorType::InvalidArgument.into());
  }
  Ok(())
}

/// Validates that pricing config ID is positive
pub fn validate_pricing_config_id(config_id: PricingConfigId) -> FastJobResult<()> {
  if config_id.0 <= 0 {
    return Err(FastJobErrorType::InvalidArgument.into());
  }
  Ok(())
}

/// Validates pricing amount is positive
pub fn validate_pricing_amount(amount: i32) -> FastJobResult<()> {
  if amount < 0 {
    return Err(FastJobErrorType::InvalidField("pricing amount must be non-negative".to_string()).into());
  }
  Ok(())
}

/// Validates pricing config name is not empty
pub fn validate_pricing_config_name(name: &str) -> FastJobResult<()> {
  if name.trim().is_empty() {
    return Err(FastJobErrorType::InvalidField("pricing config name cannot be empty".to_string()).into());
  }
  Ok(())
}

/// Validates minimum charge minutes is positive
pub fn validate_minimum_charge_minutes(minutes: i32) -> FastJobResult<()> {
  if minutes <= 0 {
    return Err(FastJobErrorType::InvalidField("minimum charge minutes must be positive".to_string()).into());
  }
  Ok(())
}

// ============================================================================
// Validated Request Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct ValidCreateCurrencyRequest(pub CreateCurrencyRequest);

impl TryFrom<CreateCurrencyRequest> for ValidCreateCurrencyRequest {
  type Error = FastJobError;

  fn try_from(value: CreateCurrencyRequest) -> Result<Self, Self::Error> {
    validate_currency_code(&value.code)?;
    validate_currency_name(&value.name)?;
    validate_currency_symbol(&value.symbol)?;
    validate_numeric_code(value.numeric_code)?;
    validate_coin_to_currency_rate(value.coin_to_currency_rate)?;
    validate_decimal_places(value.decimal_places)?;

    Ok(ValidCreateCurrencyRequest(value))
  }
}

impl TryFrom<ValidCreateCurrencyRequest> for CreateCurrency {
  type Error = FastJobError;

  fn try_from(value: ValidCreateCurrencyRequest) -> Result<Self, Self::Error> {
    Ok(CreateCurrency {
      code: value.0.code,
      name: value.0.name,
      symbol: value.0.symbol,
      numeric_code: value.0.numeric_code,
      coin_to_currency_rate: value.0.coin_to_currency_rate,
      decimal_places: value.0.decimal_places,
      thousands_separator: value.0.thousands_separator,
      decimal_separator: value.0.decimal_separator,
      symbol_position: value.0.symbol_position,
      is_default: value.0.is_default,
    })
  }
}

#[derive(Debug, Clone)]
pub struct ValidUpdateCurrencyRequest(pub UpdateCurrencyRequest);

impl TryFrom<UpdateCurrencyRequest> for ValidUpdateCurrencyRequest {
  type Error = FastJobError;

  fn try_from(value: UpdateCurrencyRequest) -> Result<Self, Self::Error> {
    validate_currency_id(value.currency_id)?;

    if let Some(ref name) = value.name {
      validate_currency_name(name)?;
    }
    if let Some(ref symbol) = value.symbol {
      validate_currency_symbol(symbol)?;
    }
    if let Some(code) = value.numeric_code {
      validate_numeric_code(code)?;
    }
    if let Some(rate) = value.coin_to_currency_rate {
      validate_coin_to_currency_rate(rate)?;
    }
    if let Some(decimal_places) = value.decimal_places {
      validate_decimal_places(decimal_places)?;
    }

    Ok(ValidUpdateCurrencyRequest(value))
  }
}

impl TryFrom<ValidUpdateCurrencyRequest> for UpdateCurrency {
  type Error = FastJobError;

  fn try_from(value: ValidUpdateCurrencyRequest) -> Result<Self, Self::Error> {
    Ok(UpdateCurrency {
      currency_id: value.0.currency_id,
      name: value.0.name,
      symbol: value.0.symbol,
      numeric_code: value.0.numeric_code,
      coin_to_currency_rate: value.0.coin_to_currency_rate,
      decimal_places: value.0.decimal_places,
      thousands_separator: value.0.thousands_separator,
      decimal_separator: value.0.decimal_separator,
      symbol_position: value.0.symbol_position,
      is_active: value.0.is_active,
      is_default: value.0.is_default,
      reason: value.0.reason,
    })
  }
}

#[derive(Debug, Clone)]
pub struct ValidCreatePricingConfigRequest(pub CreatePricingConfigRequest);

impl TryFrom<CreatePricingConfigRequest> for ValidCreatePricingConfigRequest {
  type Error = FastJobError;

  fn try_from(value: CreatePricingConfigRequest) -> Result<Self, Self::Error> {
    validate_currency_id(value.currency_id)?;
    validate_pricing_config_name(&value.name)?;
    validate_pricing_amount(value.base_fare_coin)?;
    validate_pricing_amount(value.time_charge_per_minute_coin)?;
    validate_minimum_charge_minutes(value.minimum_charge_minutes)?;
    validate_pricing_amount(value.distance_charge_per_km_coin)?;

    Ok(ValidCreatePricingConfigRequest(value))
  }
}

impl TryFrom<ValidCreatePricingConfigRequest> for CreatePricingConfig {
  type Error = FastJobError;

  fn try_from(value: ValidCreatePricingConfigRequest) -> Result<Self, Self::Error> {
    Ok(CreatePricingConfig {
      currency_id: value.0.currency_id,
      name: value.0.name,
      base_fare_coin: value.0.base_fare_coin,
      time_charge_per_minute_coin: value.0.time_charge_per_minute_coin,
      minimum_charge_minutes: value.0.minimum_charge_minutes,
      distance_charge_per_km_coin: value.0.distance_charge_per_km_coin,
      accepts_cash: value.0.accepts_cash,
      accepts_coin: value.0.accepts_coin,
      is_active: value.0.is_active,
    })
  }
}

#[derive(Debug, Clone)]
pub struct ValidUpdatePricingConfigRequest(pub UpdatePricingConfigRequest);

impl TryFrom<UpdatePricingConfigRequest> for ValidUpdatePricingConfigRequest {
  type Error = FastJobError;

  fn try_from(value: UpdatePricingConfigRequest) -> Result<Self, Self::Error> {
    validate_pricing_config_id(value.config_id)?;

    if let Some(ref name) = value.name {
      validate_pricing_config_name(name)?;
    }
    if let Some(amount) = value.base_fare_coin {
      validate_pricing_amount(amount)?;
    }
    if let Some(amount) = value.time_charge_per_minute_coin {
      validate_pricing_amount(amount)?;
    }
    if let Some(minutes) = value.minimum_charge_minutes {
      validate_minimum_charge_minutes(minutes)?;
    }
    if let Some(amount) = value.distance_charge_per_km_coin {
      validate_pricing_amount(amount)?;
    }

    Ok(ValidUpdatePricingConfigRequest(value))
  }
}

impl TryFrom<ValidUpdatePricingConfigRequest> for UpdatePricingConfig {
  type Error = FastJobError;

  fn try_from(value: ValidUpdatePricingConfigRequest) -> Result<Self, Self::Error> {
    Ok(UpdatePricingConfig {
      config_id: value.0.config_id,
      name: value.0.name,
      base_fare_coin: value.0.base_fare_coin,
      time_charge_per_minute_coin: value.0.time_charge_per_minute_coin,
      minimum_charge_minutes: value.0.minimum_charge_minutes,
      distance_charge_per_km_coin: value.0.distance_charge_per_km_coin,
      accepts_cash: value.0.accepts_cash,
      accepts_coin: value.0.accepts_coin,
      is_active: value.0.is_active,
    })
  }
}
