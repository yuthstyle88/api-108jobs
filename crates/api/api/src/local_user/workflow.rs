use actix_web::web::{Data, Json};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::{BillingId, WalletId, WorkflowId};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_wallet::api::{ApproveQuotation, ApproveWork, BillingOperationResponse, CreateInvoiceForm, CreateInvoiceResponse, DepositWallet, GetWalletResponse, SubmitStartWork, ValidCreateInvoice, WalletOperationResponse};
use lemmy_db_schema::source::wallet::{WalletModel, WalletTransactionInsertForm, TxKind};
use lemmy_utils::error::FastJobResult;

use uuid::Uuid;
use lemmy_db_schema_file::enums::WorkFlowStatus;
use lemmy_workflow::{WorkFlowOperationResponse, WorkflowService};

// Escrow-based billing workflow handlers

pub async fn create_quotation(
  data: Json<CreateInvoiceForm>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CreateInvoiceResponse>> {
  let local_user_id = local_user_view.local_user.id;

  // Validate via TryFrom into a validated wrapper
  let validated: ValidCreateInvoice = match data.into_inner().try_into() {
    Ok(v) => v,
    Err(msg) => {
      return Err(lemmy_utils::error::FastJobErrorType::InvalidField(msg).into());
    }
  };
  let data = validated.0.clone();

  // Create the invoice/billing record with detailed quotation fields
  let (billing, _quotation) = WorkflowService::create_quotation(
    &mut context.pool(),
    local_user_id,
    validated,
  ).await?;

  Ok(Json(CreateInvoiceResponse {
    billing_id: billing.id,
    issuer_id: local_user_id,
    recipient_id: data.employer_id,
    post_id: data.post_id,
    amount: data.amount,
    status: "QuotationPending".to_string(),
    delivery_timeframe_days: data.working_days,
    created_at: billing.created_at.to_rfc3339(),
    success: true,
  }))
}

pub async fn approve_quotation(
  data: Json<ApproveQuotation>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<BillingOperationResponse>> {
  let employer_id = local_user_view.local_user.id;
  let workflow_id = data.workflow_id.into();
  let wallet_id = data.wallet_id.into();
  let billing_id = data.billing_id.into();
  let wf =  WorkflowService::load_quotation_pending(&mut context.pool(),workflow_id).await?
  .approve_on(&mut context.pool(), employer_id, wallet_id, billing_id).await?;
  // Approve the quotation and convert to order

  Ok(Json(BillingOperationResponse {
    billing_id: wf.data.billing_id.unwrap_or(BillingId(0)),
    status: "OrderApproved".to_string(),
    success: true,
  }))
}

pub async fn submit_start_work(
  data: Json<SubmitStartWork>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  let worker_id = local_user_view.local_user.id;
  let workflow_id = data.workflow_id.into();
  let wf =  WorkflowService::load_order_approve(&mut context.pool(), workflow_id).await?
  .start_work_on(&mut context.pool()).await?;
  // Submit the work as the worker


  Ok(Json(WorkFlowOperationResponse {
    workflow_id: wf.data.workflow_id.into(),
    status: WorkFlowStatus::InProgress,
    success: true,
  }))
}


pub async fn submit_work(
  data: Json<ApproveWork>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {

  let workflow_id = data.workflow_id.into();
  let wf =  WorkflowService::load_in_progress(&mut context.pool(), workflow_id).await?
  .submit_work_on(&mut context.pool()).await?;
  // Approve work and release payment to worker

  Ok(Json(WorkFlowOperationResponse {
    workflow_id: wf.data.workflow_id.into(),
    status: WorkFlowStatus::WorkSubmitted,
    success: true,
  }))
}

pub async fn approve_work(
  data: Json<ApproveWork>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  let workflow_id = data.workflow_id.into();
  let wf =  WorkflowService::load_work_submit(&mut context.pool(), workflow_id).await?
  .approve_work_on(&mut context.pool()).await?;
  // Approve work and release payment to worker

  Ok(Json(WorkFlowOperationResponse {
    workflow_id: wf.data.workflow_id.into(),
    status: WorkFlowStatus::WorkSubmitted,
    success: true,
  }))
}


