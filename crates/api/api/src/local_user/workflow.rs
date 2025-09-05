use actix_web::web::{Data, Json};
use chrono::Utc;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::BillingId;
use lemmy_db_schema::source::job_budget_plan::{JobBudgetPlan, JobBudgetPlanUpdateForm};
use lemmy_db_schema::source::workflow::Workflow;
use lemmy_db_schema::source::billing::WorkStep;
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_billing::api::{
  ApproveQuotation, ApproveWork, BillingOperationResponse, CreateInvoiceForm,
  CreateInvoiceResponse, SubmitStartWork, UpdateBudgetPlanInstallments,
  UpdateBudgetPlanInstallmentsResponse, ValidCreateInvoice, ValidUpdateBudgetPlanInstallments,
  RequestRevision,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use serde_json::json;

use lemmy_db_schema_file::enums::BillingStatus;
use lemmy_db_schema_file::enums::WorkFlowStatus;
use lemmy_workflow::{WorkFlowOperationResponse, WorkflowService};

// Helper: update JobBudgetPlan.installments' status for a given workflow and seq
async fn update_job_plan_step_status(
  pool: &mut lemmy_db_schema::utils::DbPool<'_>,
  workflow_id: lemmy_db_schema::newtypes::WorkflowId,
  seq_number: i16,
  new_status: WorkFlowStatus,
) -> FastJobResult<()> {
  // Load workflow to get post_id (and authoritatve seq if needed)
  let wf_row = Workflow::read(pool, workflow_id).await?;
  let post_id = wf_row.post_id;
  let target_seq = seq_number as i32;

  // Load JobBudgetPlan by post_id
  let plan = match JobBudgetPlan::get_by_post_id(pool, post_id).await? {
    Some(p) => p,
    None => return Err(FastJobErrorType::NotFound.into()),
  };

  // Parse installments -> Vec<WorkStep>
  let mut steps: Vec<WorkStep> = serde_json::from_value(plan.installments.clone()).unwrap_or_else(|_| Vec::new());

  // Update the matching step's status
  let mut found = false;
  for s in &mut steps {
    if s.seq == target_seq {
      s.status = new_status;
      found = true;
      break;
    }
  }
  if !found {
    // If not found, we won't fail hard; just return OK to avoid blocking WF progression
    return Ok(());
  }

  let phases_json = serde_json::to_value(&steps).unwrap_or_else(|_| json!([]));
  let update = JobBudgetPlanUpdateForm {
    installments: Some(phases_json),
    updated_at: Some(Some(Utc::now())),
    ..Default::default()
  };
  let _ = JobBudgetPlan::update(pool, plan.id, &update).await?;
  Ok(())
}

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
      return Err(FastJobErrorType::InvalidField(msg).into());
    }
  };
  let data = validated.0.clone();

  // Create the invoice/billing record with detailed quotation fields
  let (billing, _quotation) =
    WorkflowService::create_quotation(&mut context.pool(), local_user_id, validated).await?;

  Ok(Json(CreateInvoiceResponse {
    billing_id: billing.id,
    issuer_id: local_user_id,
    recipient_id: data.employer_id,
    post_id: data.post_id,
    amount: data.amount,
    status: BillingStatus::QuotePendingReview,
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
  let employer_id = local_user_view.local_user.person_id;
  let workflow_id = data.workflow_id.into();
  let wallet_id = data.wallet_id.into();
  let billing_id = data.billing_id.into();
  let wf = WorkflowService::load_quotation_pending(&mut context.pool(), workflow_id)
    .await?
    .approve_on(&mut context.pool(), employer_id, wallet_id, billing_id)
    .await?;
  // Approve the quotation and convert to order

  Ok(Json(BillingOperationResponse {
    billing_id: wf.data.billing_id.unwrap_or(BillingId(0)),
    status: BillingStatus::OrderApproved,
    success: true,
  }))
}

pub async fn submit_start_work(
  data: Json<SubmitStartWork>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  let _worker_id = local_user_view.local_user.id;
  let workflow_id = data.workflow_id.into();
  let seq_number = data.seq_number; // use provided seq for plan update
  let wf = WorkflowService::load_order_approve(&mut context.pool(), workflow_id)
    .await?
    .start_work_on(&mut context.pool())
    .await?;

  // Update JobBudgetPlan step status -> InProgress for this seq
  update_job_plan_step_status(
    &mut context.pool(),
    workflow_id,
    seq_number,
    WorkFlowStatus::InProgress,
  )
  .await?;

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
  let seq_number = data.seq_number;
  let wf = WorkflowService::load_in_progress(&mut context.pool(), workflow_id)
    .await?
    .submit_work_on(&mut context.pool())
    .await?;

  // Update JobBudgetPlan step status -> PendingEmployerReview for this seq
  update_job_plan_step_status(
    &mut context.pool(),
    workflow_id,
    seq_number,
    WorkFlowStatus::PendingEmployerReview,
  )
  .await?;

  Ok(Json(WorkFlowOperationResponse {
    workflow_id: wf.data.workflow_id.into(),
    status: WorkFlowStatus::PendingEmployerReview,
    success: true,
  }))
}

pub async fn approve_work(
  data: Json<ApproveWork>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  let workflow_id = data.workflow_id.into();
  let seq_number = data.seq_number;
  let site_view = context.site_config().get().await?.site_view;
  let coin_id = site_view
    .clone()
    .local_site
    .coin_id
    .ok_or_else(|| anyhow::anyhow!("Coin ID not set"))?;
  let platform_wallet_id = context
    .site_config()
    .get()
    .await?
    .admins
    .first()
    .unwrap()
    .person
    .wallet_id;

  let wf = WorkflowService::load_work_submit(&mut context.pool(), workflow_id)
    .await?
    .approve_work_on(&mut context.pool(), coin_id, platform_wallet_id)
    .await?;

  // Update JobBudgetPlan step status -> Completed for this seq
  update_job_plan_step_status(
    &mut context.pool(),
    workflow_id,
    seq_number,
    WorkFlowStatus::Completed,
  )
  .await?;

  Ok(Json(WorkFlowOperationResponse {
    workflow_id: wf.data.workflow_id.into(),
    status: WorkFlowStatus::PendingEmployerReview,
    success: true,
  }))
}

pub async fn request_revision(
  data: Json<RequestRevision>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  let workflow_id = data.workflow_id.into();
  let seq_number = data.seq_number;
  let _reason = data.reason.clone();

  let wf = WorkflowService::load_work_submit(&mut context.pool(), workflow_id)
    .await?
    .request_revision_on(&mut context.pool())
    .await?;

  // Update JobBudgetPlan step status -> InProgress for this seq
  update_job_plan_step_status(
    &mut context.pool(),
    workflow_id,
    seq_number,
    WorkFlowStatus::InProgress,
  )
  .await?;

  Ok(Json(WorkFlowOperationResponse {
    workflow_id: wf.data.workflow_id.into(),
    status: WorkFlowStatus::InProgress,
    success: true,
  }))
}

/// Replace the installments array for a job budget plan using items like
/// [{ "idx": 1, "amount": 500000, "status": "paid" }, ...]
pub async fn update_budget_plan_status(
  data: Json<UpdateBudgetPlanInstallments>,
  context: Data<FastJobContext>,
  _local_user_view: LocalUserView,
) -> FastJobResult<Json<UpdateBudgetPlanInstallmentsResponse>> {
  let post_id = data.post_id;

  // Validate via TryFrom into a validated wrapper
  let validated: ValidUpdateBudgetPlanInstallments = data.into_inner().try_into()?;

  // Load plan
  let plan = match JobBudgetPlan::get_by_post_id(&mut context.pool(), post_id).await? {
    Some(p) => p,
    None => {
      return Err(FastJobErrorType::NotFound.into());
    }
  };

  // Ensure sorted by seq to keep order by seq_number
  let mut items = validated.0.installments.clone();
  items.sort_by_key(|w| w.seq);
  let phases_json = serde_json::to_value(&items).unwrap_or_else(|_| json!([]));

  let update = JobBudgetPlanUpdateForm {
    installments: Some(phases_json),
    updated_at: Some(Some(Utc::now())),
    ..Default::default()
  };

  let updated = JobBudgetPlan::update(&mut context.pool(), plan.id, &update).await?;

  Ok(Json(UpdateBudgetPlanInstallmentsResponse {
    budget_plan: updated,
    success: true,
  }))
}
