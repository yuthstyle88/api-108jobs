//! Validation logic for wallet-related requests
use crate::api::{SubmitWithdrawRequest, UpdateWallet, UpdateWalletRequest};
use app_108jobs_db_schema::newtypes::{BankAccountId, Coin, CurrencyId, WalletId};
use app_108jobs_utils::error::{FastJobError, FastJobErrorType, FastJobResult};

/// Validates that a wallet ID is positive
pub fn validate_wallet_id(wallet_id: WalletId) -> FastJobResult<()> {
  if wallet_id.0 <= 0 {
    return Err(FastJobErrorType::InvalidArgument.into());
  }
  Ok(())
}

/// Validates that a bank account ID is positive
pub fn validate_bank_account_id(bank_account_id: BankAccountId) -> FastJobResult<()> {
  if bank_account_id.0 <= 0 {
    return Err(FastJobErrorType::InvalidArgument.into());
  }
  Ok(())
}

/// Validates that a currency ID is positive
pub fn validate_currency_id(currency_id: CurrencyId) -> FastJobResult<()> {
  if currency_id.0 <= 0 {
    return Err(FastJobErrorType::InvalidArgument.into());
  }
  Ok(())
}

/// Validates that an amount is positive
pub fn validate_amount_positive(amount: Coin) -> FastJobResult<()> {
  if amount <= 0 {
    return Err(FastJobErrorType::AmountMustBePositive.into());
  }
  Ok(())
}

/// Validates that a reason string is not empty
pub fn validate_reason_not_empty(reason: &str) -> FastJobResult<()> {
  if reason.trim().is_empty() {
    return Err(FastJobErrorType::InvalidArgument.into());
  }
  Ok(())
}

/// Validates that a reason string does not exceed max length
pub const MAX_REASON_LENGTH: usize = 500;

pub fn validate_reason_length(reason: &str) -> FastJobResult<()> {
  if reason.len() > MAX_REASON_LENGTH {
    return Err(FastJobErrorType::InvalidLength.into());
  }
  Ok(())
}

/// Validates a reason string (not empty and within max length)
pub fn validate_reason(reason: &str) -> FastJobResult<()> {
  validate_reason_not_empty(reason)?;
  validate_reason_length(reason)?;
  Ok(())
}

// ============================================================================
// Validated Request Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct ValidUpdateWalletRequest(pub UpdateWalletRequest);

impl TryFrom<UpdateWalletRequest> for ValidUpdateWalletRequest {
  type Error = FastJobError;

  fn try_from(value: UpdateWalletRequest) -> Result<Self, Self::Error> {
    validate_amount_positive(value.amount)?;
    Ok(ValidUpdateWalletRequest(value))
  }
}

impl TryFrom<ValidUpdateWalletRequest> for UpdateWallet {
  type Error = FastJobError;

  fn try_from(value: ValidUpdateWalletRequest) -> Result<Self, Self::Error> {
    Ok(UpdateWallet {
      amount: value.0.amount,
    })
  }
}

#[derive(Debug, Clone)]
pub struct ValidSubmitWithdrawRequest(pub SubmitWithdrawRequest);

impl TryFrom<SubmitWithdrawRequest> for ValidSubmitWithdrawRequest {
  type Error = FastJobError;

  fn try_from(value: SubmitWithdrawRequest) -> Result<Self, Self::Error> {
    // Validate all fields
    validate_wallet_id(value.wallet_id)?;
    validate_bank_account_id(value.bank_account_id)?;
    validate_amount_positive(value.amount)?;
    validate_currency_id(value.currency_id)?;
    validate_reason(&value.reason)?;

    Ok(ValidSubmitWithdrawRequest(value))
  }
}
