//! Validation logic for rider requests
use crate::api::{CreateRider, CreateRiderRequest};
use app_108jobs_utils::error::{FastJobError, FastJobResult};
use chrono::{DateTime, Utc};

/// Validates that license expiry date is in the future
pub fn validate_license_expiry(expiry_date: Option<DateTime<Utc>>) -> FastJobResult<()> {
  if let Some(date) = expiry_date {
    if date <= Utc::now() {
      return Err(app_108jobs_utils::error::FastJobErrorType::InvalidField(
        "license expiry date must be in the future".to_string(),
      )
      .into());
    }
  }
  Ok(())
}

/// Validates vehicle plate number if provided
pub fn validate_plate_number(plate_number: Option<&String>) -> FastJobResult<()> {
  if let Some(plate) = plate_number {
    if plate.trim().is_empty() {
      return Err(app_108jobs_utils::error::FastJobErrorType::InvalidField(
        "plate number cannot be empty".to_string(),
      )
      .into());
    }
  }
  Ok(())
}

/// Validates license number if provided
pub fn validate_license_number(license_number: Option<&String>) -> FastJobResult<()> {
  if let Some(license) = license_number {
    if license.trim().is_empty() {
      return Err(app_108jobs_utils::error::FastJobErrorType::InvalidField(
        "license number cannot be empty".to_string(),
      )
      .into());
    }
  }
  Ok(())
}

// ============================================================================
// Validated Request Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct ValidCreateRiderRequest(pub CreateRiderRequest);

impl TryFrom<CreateRiderRequest> for ValidCreateRiderRequest {
  type Error = FastJobError;

  fn try_from(value: CreateRiderRequest) -> Result<Self, Self::Error> {
    validate_license_expiry(value.license_expiry_date)?;
    validate_plate_number(value.vehicle_plate_number.as_ref())?;
    validate_license_number(value.license_number.as_ref())?;

    Ok(ValidCreateRiderRequest(value))
  }
}

#[derive(Debug, Clone)]
pub struct ValidCreateRider(pub CreateRider);

impl TryFrom<CreateRiderRequest> for ValidCreateRider {
  type Error = FastJobError;

  fn try_from(value: CreateRiderRequest) -> Result<Self, Self::Error> {
    validate_license_expiry(value.license_expiry_date)?;
    validate_plate_number(value.vehicle_plate_number.as_ref())?;
    validate_license_number(value.license_number.as_ref())?;

    Ok(ValidCreateRider(CreateRider {
      vehicle_type: value.vehicle_type,
      vehicle_plate_number: value.vehicle_plate_number,
      license_number: value.license_number,
      license_expiry_date: value.license_expiry_date,
    }))
  }
}
