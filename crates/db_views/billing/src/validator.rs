//! Validation logic for billing/workflow requests
use crate::api::{
  ApproveQuotationForm, ApproveQuotationRequest, ApproveWorkForm, ApproveWorkRequest,
  CancelJobForm, CancelJobRequest, CreateInvoiceForm, CreateInvoiceRequest,
  RequestRevisionForm, RequestRevisionRequest, StartWorkflowForm, StartWorkflowRequest,
  SubmitStartWorkForm, SubmitStartWorkRequest, UpdateBudgetPlanInstallments,
  UpdateBudgetPlanInstallmentsRequest,
};
use app_108jobs_db_schema::newtypes::Coin;
use app_108jobs_db_schema::source::billing::WorkStep;
use app_108jobs_utils::error::{FastJobError, FastJobErrorType, FastJobResult};

/// Validates that an amount is positive
pub fn validate_amount_positive(amount: Coin) -> FastJobResult<()> {
  if amount <= 0 {
    return Err(FastJobErrorType::AmountMustBePositive.into());
  }
  Ok(())
}

/// Validates that a sequence number is positive
pub fn validate_seq_number_positive(seq_number: i16) -> FastJobResult<()> {
  if seq_number <= 0 {
    return Err(FastJobErrorType::InvalidArgument.into());
  }
  Ok(())
}

/// Validates that work description is not empty
pub fn validate_work_description_not_empty(description: &str) -> FastJobResult<()> {
  if description.trim().is_empty() {
    return Err(FastJobErrorType::InvalidArgument.into());
  }
  Ok(())
}

/// Validates that all work steps have non-negative amounts
pub fn validate_work_steps_amounts(steps: &[WorkStep]) -> FastJobResult<()> {
  for step in steps {
    if step.amount < 0 {
      return Err(FastJobErrorType::NegativeAmount.into());
    }
  }
  Ok(())
}

// ============================================================================
// Validated Request Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct ValidCreateInvoiceRequest(pub CreateInvoiceRequest);

impl TryFrom<CreateInvoiceRequest> for ValidCreateInvoiceRequest {
  type Error = FastJobError;

  fn try_from(value: CreateInvoiceRequest) -> Result<Self, Self::Error> {
    validate_amount_positive(value.amount)?;
    validate_seq_number_positive(value.seq_number)?;
    Ok(ValidCreateInvoiceRequest(value))
  }
}

impl TryFrom<ValidCreateInvoiceRequest> for CreateInvoiceForm {
  type Error = FastJobError;

  fn try_from(value: ValidCreateInvoiceRequest) -> Result<Self, Self::Error> {
    Ok(CreateInvoiceForm {
      employer_id: value.0.employer_id,
      post_id: value.0.post_id,
      comment_id: value.0.comment_id,
      seq_number: value.0.seq_number,
      amount: value.0.amount,
      proposal: value.0.proposal,
      project_name: value.0.project_name,
      status: value.0.status,
      project_details: value.0.project_details,
      working_days: value.0.working_days,
      deliverables: value.0.deliverables,
      note: value.0.note,
      starting_day: value.0.starting_day,
      delivery_day: value.0.delivery_day,
      room_id: value.0.room_id,
      workflow_id: value.0.workflow_id,
    })
  }
}

#[derive(Debug, Clone)]
pub struct ValidApproveQuotationRequest(pub ApproveQuotationRequest);

impl TryFrom<ApproveQuotationRequest> for ValidApproveQuotationRequest {
  type Error = FastJobError;

  fn try_from(value: ApproveQuotationRequest) -> Result<Self, Self::Error> {
    validate_seq_number_positive(value.seq_number)?;
    Ok(ValidApproveQuotationRequest(value))
  }
}

impl TryFrom<ValidApproveQuotationRequest> for ApproveQuotationForm {
  type Error = FastJobError;

  fn try_from(value: ValidApproveQuotationRequest) -> Result<Self, Self::Error> {
    Ok(ApproveQuotationForm {
      seq_number: value.0.seq_number,
      billing_id: value.0.billing_id,
      wallet_id: value.0.wallet_id,
      workflow_id: value.0.workflow_id,
    })
  }
}

#[derive(Debug, Clone)]
pub struct ValidStartWorkflowRequest(pub StartWorkflowRequest);

impl TryFrom<StartWorkflowRequest> for ValidStartWorkflowRequest {
  type Error = FastJobError;

  fn try_from(value: StartWorkflowRequest) -> Result<Self, Self::Error> {
    validate_seq_number_positive(value.seq_number)?;
    Ok(ValidStartWorkflowRequest(value))
  }
}

impl TryFrom<ValidStartWorkflowRequest> for StartWorkflowForm {
  type Error = FastJobError;

  fn try_from(value: ValidStartWorkflowRequest) -> Result<Self, Self::Error> {
    Ok(StartWorkflowForm {
      post_id: value.0.post_id,
      seq_number: value.0.seq_number,
      room_id: value.0.room_id,
    })
  }
}

#[derive(Debug, Clone)]
pub struct ValidSubmitStartWorkRequest(pub SubmitStartWorkRequest);

impl TryFrom<SubmitStartWorkRequest> for ValidSubmitStartWorkRequest {
  type Error = FastJobError;

  fn try_from(value: SubmitStartWorkRequest) -> Result<Self, Self::Error> {
    validate_seq_number_positive(value.seq_number)?;
    validate_work_description_not_empty(&value.work_description)?;
    Ok(ValidSubmitStartWorkRequest(value))
  }
}

impl TryFrom<ValidSubmitStartWorkRequest> for SubmitStartWorkForm {
  type Error = FastJobError;

  fn try_from(value: ValidSubmitStartWorkRequest) -> Result<Self, Self::Error> {
    Ok(SubmitStartWorkForm {
      seq_number: value.0.seq_number,
      workflow_id: value.0.workflow_id,
      work_description: value.0.work_description,
      deliverable_url: value.0.deliverable_url,
    })
  }
}

#[derive(Debug, Clone)]
pub struct ValidApproveWorkRequest(pub ApproveWorkRequest);

impl TryFrom<ApproveWorkRequest> for ValidApproveWorkRequest {
  type Error = FastJobError;

  fn try_from(value: ApproveWorkRequest) -> Result<Self, Self::Error> {
    validate_seq_number_positive(value.seq_number)?;
    Ok(ValidApproveWorkRequest(value))
  }
}

impl TryFrom<ValidApproveWorkRequest> for ApproveWorkForm {
  type Error = FastJobError;

  fn try_from(value: ValidApproveWorkRequest) -> Result<Self, Self::Error> {
    Ok(ApproveWorkForm {
      seq_number: value.0.seq_number,
      workflow_id: value.0.workflow_id,
      room_id: value.0.room_id,
      billing_id: value.0.billing_id,
    })
  }
}

#[derive(Debug, Clone)]
pub struct ValidRequestRevisionRequest(pub RequestRevisionRequest);

impl TryFrom<RequestRevisionRequest> for ValidRequestRevisionRequest {
  type Error = FastJobError;

  fn try_from(value: RequestRevisionRequest) -> Result<Self, Self::Error> {
    validate_seq_number_positive(value.seq_number)?;
    Ok(ValidRequestRevisionRequest(value))
  }
}

impl TryFrom<ValidRequestRevisionRequest> for RequestRevisionForm {
  type Error = FastJobError;

  fn try_from(value: ValidRequestRevisionRequest) -> Result<Self, Self::Error> {
    Ok(RequestRevisionForm {
      seq_number: value.0.seq_number,
      workflow_id: value.0.workflow_id,
      reason: value.0.reason,
    })
  }
}

#[derive(Debug, Clone)]
pub struct ValidUpdateBudgetPlanInstallmentsRequest(pub UpdateBudgetPlanInstallmentsRequest);

impl TryFrom<UpdateBudgetPlanInstallmentsRequest> for ValidUpdateBudgetPlanInstallmentsRequest {
  type Error = FastJobError;

  fn try_from(value: UpdateBudgetPlanInstallmentsRequest) -> Result<Self, Self::Error> {
    validate_work_steps_amounts(&value.installments)?;
    Ok(ValidUpdateBudgetPlanInstallmentsRequest(value))
  }
}

impl TryFrom<ValidUpdateBudgetPlanInstallmentsRequest> for UpdateBudgetPlanInstallments {
  type Error = FastJobError;

  fn try_from(value: ValidUpdateBudgetPlanInstallmentsRequest) -> Result<Self, Self::Error> {
    Ok(UpdateBudgetPlanInstallments {
      post_id: value.0.post_id,
      installments: value.0.installments,
    })
  }
}

#[derive(Debug, Clone)]
pub struct ValidCancelJobRequest(pub CancelJobRequest);

impl TryFrom<CancelJobRequest> for ValidCancelJobRequest {
  type Error = FastJobError;

  fn try_from(value: CancelJobRequest) -> Result<Self, Self::Error> {
    validate_seq_number_positive(value.seq_number)?;
    Ok(ValidCancelJobRequest(value))
  }
}

impl TryFrom<ValidCancelJobRequest> for CancelJobForm {
  type Error = FastJobError;

  fn try_from(value: ValidCancelJobRequest) -> Result<Self, Self::Error> {
    Ok(CancelJobForm {
      seq_number: value.0.seq_number,
      workflow_id: value.0.workflow_id,
      reason: value.0.reason,
      current_status: value.0.current_status,
    })
  }
}
