//! Validation logic for bank account requests
use crate::api::{
  BankAccountForm, CreateBankAccount, CreateBankAccountRequest, DeleteBankAccount,
  DeleteBankAccountRequest, UpdateBankAccount, UpdateBankAccountRequest,
};
use app_108jobs_db_schema::newtypes::{BankAccountId, BankId};
use app_108jobs_utils::error::{FastJobError, FastJobErrorType, FastJobResult};
use app_108jobs_utils::utils::validation::validate_bank_account;

/// Validates bank account number by country
pub fn validate_account_number(country_id: &str, account_number: &str) -> FastJobResult<()> {
  let acc_num = account_number.trim();
  if acc_num.is_empty() || !validate_bank_account(country_id, acc_num) {
    return Err(FastJobErrorType::InvalidAccountNumber.into());
  }
  Ok(())
}

/// Validates account name is not empty
pub fn validate_account_name(account_name: &str) -> FastJobResult<()> {
  if account_name.trim().is_empty() {
    return Err(FastJobErrorType::InvalidAccountName.into());
  }
  Ok(())
}

/// Validates bank ID is positive
pub fn validate_bank_id(bank_id: BankId) -> FastJobResult<()> {
  if bank_id.0 <= 0 {
    return Err(FastJobErrorType::InvalidArgument.into());
  }
  Ok(())
}

/// Validates bank account ID is positive
pub fn validate_bank_account_id(bank_account_id: BankAccountId) -> FastJobResult<()> {
  if bank_account_id.0 <= 0 {
    return Err(FastJobErrorType::InvalidArgument.into());
  }
  Ok(())
}

// ============================================================================
// Validated Request Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct ValidCreateBankAccountRequest(pub CreateBankAccountRequest);

impl TryFrom<CreateBankAccountRequest> for ValidCreateBankAccountRequest {
  type Error = FastJobError;

  fn try_from(value: CreateBankAccountRequest) -> Result<Self, Self::Error> {
    validate_bank_id(value.bank_id)?;
    validate_account_number(&value.country_id, &value.account_number)?;
    validate_account_name(&value.account_name)?;

    Ok(ValidCreateBankAccountRequest(value))
  }
}

impl TryFrom<ValidCreateBankAccountRequest> for CreateBankAccount {
  type Error = FastJobError;

  fn try_from(value: ValidCreateBankAccountRequest) -> Result<Self, Self::Error> {
    Ok(CreateBankAccount {
      bank_id: value.0.bank_id,
      account_number: value.0.account_number,
      account_name: value.0.account_name,
      is_default: value.0.is_default,
      verification_image: value.0.verification_image,
    })
  }
}

impl TryFrom<ValidCreateBankAccountRequest> for BankAccountForm {
  type Error = FastJobError;

  fn try_from(value: ValidCreateBankAccountRequest) -> Result<Self, Self::Error> {
    Ok(BankAccountForm {
      bank_id: value.0.bank_id,
      account_number: value.0.account_number,
      account_name: value.0.account_name,
      is_default: value.0.is_default,
      country_id: value.0.country_id,
      verification_image: value.0.verification_image,
    })
  }
}

#[derive(Debug, Clone)]
pub struct ValidUpdateBankAccountRequest(pub UpdateBankAccountRequest);

impl TryFrom<UpdateBankAccountRequest> for ValidUpdateBankAccountRequest {
  type Error = FastJobError;

  fn try_from(value: UpdateBankAccountRequest) -> Result<Self, Self::Error> {
    validate_bank_account_id(value.bank_account_id)?;

    if let Some(bank_id) = value.bank_id {
      validate_bank_id(bank_id)?;
    }
    if let Some(ref account_number) = value.account_number {
      // Note: can't validate account_number without country_id, which is not in UpdateBankAccountRequest
      if account_number.trim().is_empty() {
        return Err(FastJobErrorType::InvalidAccountNumber.into());
      }
    }
    if let Some(ref account_name) = value.account_name {
      validate_account_name(account_name)?;
    }

    Ok(ValidUpdateBankAccountRequest(value))
  }
}

impl TryFrom<ValidUpdateBankAccountRequest> for UpdateBankAccount {
  type Error = FastJobError;

  fn try_from(value: ValidUpdateBankAccountRequest) -> Result<Self, Self::Error> {
    Ok(UpdateBankAccount {
      bank_account_id: value.0.bank_account_id,
      bank_id: value.0.bank_id,
      account_number: value.0.account_number,
      account_name: value.0.account_name,
      is_default: value.0.is_default,
      verification_image: value.0.verification_image,
    })
  }
}

#[derive(Debug, Clone)]
pub struct ValidDeleteBankAccountRequest(pub DeleteBankAccountRequest);

impl TryFrom<DeleteBankAccountRequest> for ValidDeleteBankAccountRequest {
  type Error = FastJobError;

  fn try_from(value: DeleteBankAccountRequest) -> Result<Self, Self::Error> {
    validate_bank_account_id(value.bank_account_id)?;
    Ok(ValidDeleteBankAccountRequest(value))
  }
}

impl TryFrom<ValidDeleteBankAccountRequest> for DeleteBankAccount {
  type Error = FastJobError;

  fn try_from(value: ValidDeleteBankAccountRequest) -> Result<Self, Self::Error> {
    Ok(DeleteBankAccount {
      bank_account_id: value.0.bank_account_id,
    })
  }
}
