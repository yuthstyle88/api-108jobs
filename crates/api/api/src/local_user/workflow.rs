use actix_web::web::{Data, Json, Query};
use chrono::Utc;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::billing::Billing;
use lemmy_db_schema::source::billing::WorkStep;
use lemmy_db_schema::source::job_budget_plan::{JobBudgetPlan, JobBudgetPlanUpdateForm};
use lemmy_db_schema::source::workflow::{Workflow, WorkflowUpdateForm};
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_billing::api::{ApproveQuotationForm, ApproveWorkForm, CancelJobForm, CreateInvoiceForm, CreateInvoiceResponse, GetBillingByCommentQuery, RequestRevisionForm, StartWorkflowForm, SubmitStartWorkForm, UpdateBudgetPlanInstallments, UpdateBudgetPlanInstallmentsResponse, ValidApproveQuotation, ValidApproveWork, ValidCancelJob, ValidCreateInvoice, ValidRequestRevision, ValidStartWorkflow, ValidSubmitStartWork, ValidUpdateBudgetPlanInstallments};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use serde_json::json;
use lemmy_db_schema_file::enums::BillingStatus;
use lemmy_db_schema_file::enums::WorkFlowStatus;
use lemmy_workflow::{WorkFlowOperationResponse, WorkflowService};

// Helper: update JobBudgetPlan.installments' status for a given workflow and seq
async fn _update_job_plan_step_status(
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
  let mut steps: Vec<WorkStep> = serde_json::from_value(plan.installments.clone())
    .map_err(|_| FastJobErrorType::InvalidField("Invalid installments JSON".into()))?;

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

  let phases_json = serde_json::to_value(&steps)
    .map_err(|_| FastJobErrorType::InvalidField("Invalid installments serialization".into()))?;
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
  // Validate via TryFrom into a validated wrapper
  let validated: ValidCreateInvoice = match data.into_inner().try_into() {
    Ok(v) => v,
    Err(msg) => {
      return Err(FastJobErrorType::InvalidField(msg).into());
    }
  };
  let data = validated.0.clone();

  // Create the invoice/billing record with detailed quotation fields
  let billing =
    WorkflowService::create_quotation(&mut context.pool(), local_user_view.local_user.id, validated)
      .await?;

  Ok(Json(CreateInvoiceResponse {
    billing_id: billing.id,
    issuer_id: local_user_view.local_user.id,
    recipient_id: data.employer_id,
    post_id: data.post_id,
    amount: data.amount,
    status: billing.status,
    delivery_timeframe_days: data.working_days,
    created_at: billing.created_at.to_rfc3339(),
    success: true,
  }))
}

pub async fn approve_quotation(
  data: Json<ApproveQuotationForm>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  let employer_id = local_user_view.local_user.id;

  // Validate
  let validated: ValidApproveQuotation = match data.into_inner().try_into() {
    Ok(v) => v,
    Err(msg) => return Err(FastJobErrorType::InvalidField(msg).into()),
  };
  let form = validated.0;
  let wf = WorkflowService::load_quotation_pending(&mut context.pool(), form.workflow_id)
    .await?
    .approve_on(
      &mut context.pool(),
      employer_id,
      form.wallet_id,
      form.billing_id,
    )
    .await?;
  // Approve the quotation and convert to order

  Ok(Json(WorkFlowOperationResponse {
    workflow_id: wf.data.workflow_id.into(),
    status: WorkFlowStatus::OrderApproved,
    success: true,
  }))
}

pub async fn submit_start_work(
  data: Json<SubmitStartWorkForm>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  // Validate input
  let validated: ValidSubmitStartWork = match data.into_inner().try_into() {
    Ok(v) => v,
    Err(msg) => return Err(FastJobErrorType::InvalidField(msg).into()),
  };
  let form = validated.0;

  // Apply transition: OrderApproved -> InProgress
  let wf = WorkflowService::load_order_approve(&mut context.pool(), form.workflow_id)
    .await?
    .start_work_on(&mut context.pool())
    .await?;

  Ok(Json(WorkFlowOperationResponse {
    workflow_id: wf.data.workflow_id.into(),
    status: WorkFlowStatus::InProgress,
    success: true,
  }))
}

pub async fn submit_work(
  data: Json<SubmitStartWorkForm>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  // Validate input
  let validated: ValidSubmitStartWork = match data.into_inner().try_into() {
    Ok(v) => v,
    Err(msg) => return Err(FastJobErrorType::InvalidField(msg).into()),
  };
  let form = validated.0;

  // Apply transition: InProgress -> PendingEmployerReview
  let wf = WorkflowService::load_in_progress(&mut context.pool(), form.workflow_id)
    .await?
    .submit_work_on(&mut context.pool())
    .await?;

  // Save submitted work content
  let _ = Workflow::update(
    &mut context.pool(),
    form.workflow_id,
    &WorkflowUpdateForm {
      deliverable_accepted: Some(false),
      deliverable_submitted_at: Some(Some(Utc::now())),
      deliverable_url: Some(form.deliverable_url),
      updated_at: Some(Some(Utc::now())),
      ..Default::default()
    },
  )
  .await?;

  Ok(Json(WorkFlowOperationResponse {
    workflow_id: wf.data.workflow_id.into(),
    status: WorkFlowStatus::PendingEmployerReview,
    success: true,
  }))
}

pub async fn approve_work(
  data: Json<ApproveWorkForm>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  // Validate input
  let validated: ValidApproveWork = match data.into_inner().try_into() {
    Ok(v) => v,
    Err(msg) => return Err(FastJobErrorType::InvalidField(msg).into()),
  };
  let form = validated.0;
  let workflow_id = form.workflow_id;
  let comment_id = form.comment_id;
  let site_view = context.site_config().get().await?.site_view;
  let coin_id = site_view
    .clone()
    .local_site
    .coin_id
    .ok_or_else(|| anyhow::anyhow!("Coin ID not set"))?;
  let platform_wallet_id = match context.site_config().get().await?.admins.first() {
    Some(a) => a.person.wallet_id,
    None => {
      return Err(FastJobErrorType::InvalidField("No platform admin configured".into()).into());
    }
  };

  let wf = WorkflowService::load_work_submit(&mut context.pool(), workflow_id)
    .await?
    .approve_work_on(&mut context.pool(), coin_id, platform_wallet_id, comment_id)
    .await?;

  Ok(Json(WorkFlowOperationResponse {
    workflow_id: wf.data.workflow_id.into(),
    status: WorkFlowStatus::Completed,
    success: true,
  }))
}

pub async fn request_revision(
  data: Json<RequestRevisionForm>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  // Validate input
  let validated: ValidRequestRevision = match data.into_inner().try_into() {
    Ok(v) => v,
    Err(msg) => return Err(FastJobErrorType::InvalidField(msg).into()),
  };
  let form = validated.0;
  let workflow_id = form.workflow_id;

  let wf = WorkflowService::load_work_submit(&mut context.pool(), workflow_id)
    .await?
    .request_revision_on(&mut context.pool(), form.reason.clone())
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

pub async fn start_workflow(
  data: Json<StartWorkflowForm>,
  context: Data<FastJobContext>,
  _local_user_view: LocalUserView,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  // Validate input
  let validated: ValidStartWorkflow = match data.into_inner().try_into() {
    Ok(v) => v,
    Err(msg) => return Err(FastJobErrorType::InvalidField(msg).into()),
  };
  let form = validated.0;
  let wf = WorkflowService::start_workflow(
    &mut context.pool(),
    form.post_id,
    form.seq_number,
    form.room_id,
  )
  .await?;

  Ok(Json(WorkFlowOperationResponse {
    workflow_id: wf.id.into(),
    status: WorkFlowStatus::QuotationPending,
    success: true,
  }))
}

pub async fn cancel_job(
  data: Json<CancelJobForm>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  // Validate input
  let validated: ValidCancelJob = match data.into_inner().try_into() {
    Ok(v) => v,
    Err(msg) => return Err(FastJobErrorType::InvalidField(msg).into()),
  };
  let form = validated.0;

  // Perform cancellation (allowed for any non-finalized status)
  WorkflowService::cancel(&mut context.pool(), form.workflow_id).await?;

  Ok(Json(WorkFlowOperationResponse {
    workflow_id: form.workflow_id.into(),
    status: WorkFlowStatus::Cancelled,
    success: true,
  }))
}

/// GET billing by comment id where status is QuotePendingReview
pub async fn get_billing_by_comment(
  query: Query<GetBillingByCommentQuery>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<Billing>> {
  let mut pool = context.pool();
  let comment_id = query.comment_id;
  let bill_opt =
    Billing::get_by_comment_and_status(&mut pool, comment_id, BillingStatus::QuotePendingReview)
      .await?;

  match bill_opt {
    Some(b) => {
      let pid = local_user_view.local_user.id;
      if b.freelancer_id == pid || b.employer_id == pid {
        Ok(Json(b))
      } else {
        Err(FastJobErrorType::NotAllowed.into())
      }
    }
    None => Err(FastJobErrorType::NotFound.into()),
  }
}
