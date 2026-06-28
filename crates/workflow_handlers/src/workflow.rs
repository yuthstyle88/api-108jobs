use crate::workflow_authz::{require_any_party, require_post_creator, require_role, WorkflowRole};
use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_core::error::{FastJobErrorType, FastJobResult};
use app_108jobs_db::{
  enums::{BillingStatus, WorkFlowStatus},
  newtypes::ChatRoomId,
  source::{
    billing::{Billing, WorkStep},
    chat_participant::ChatParticipant,
    job_budget_plan::{JobBudgetPlan, JobBudgetPlanUpdateForm},
    post::Post,
    workflow::{Workflow, WorkflowUpdateForm},
  },
  traits::Crud,
};
use app_108jobs_db_views_billing::{
  api::{
    ApproveQuotationRequest,
    ApproveWorkRequest,
    CancelJobRequest,
    CreateInvoiceRequest,
    RequestRevisionRequest,
    StartWorkflowRequest,
    SubmitStartWorkRequest,
    UpdateBudgetPlanInstallmentsRequest,
  },
  CreateInvoiceResponse,
  GetBillingByRoomQuery,
  UpdateBudgetPlanInstallmentsResponse,
  ValidApproveQuotationRequest,
  ValidApproveWorkRequest,
  ValidCancelJobRequest,
  ValidCreateInvoiceRequest,
  ValidRequestRevisionRequest,
  ValidStartWorkflowRequest,
  ValidSubmitStartWorkRequest,
  ValidUpdateBudgetPlanInstallmentsRequest,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_workflow::{WorkFlowOperationResponse, WorkflowService};
use chrono::Utc;
use serde_json::json;

/// Load the `Billing` row backing a workflow, or `NotFound` if none exists yet.
async fn billing_for_workflow(
  pool: &mut app_108jobs_db::utils::DbPool<'_>,
  workflow_id: app_108jobs_db::newtypes::WorkflowId,
) -> FastJobResult<Billing> {
  let wf = Workflow::read(pool, workflow_id).await?;
  let billing_id = match wf.billing_id {
    Some(id) => id,
    None => return Err(FastJobErrorType::NotFound.into()),
  };
  Billing::read(pool, billing_id).await
}

/// True if `caller` is a participant of `room_id`.
async fn is_room_participant(
  pool: &mut app_108jobs_db::utils::DbPool<'_>,
  room_id: ChatRoomId,
  caller: app_108jobs_db::newtypes::LocalUserId,
) -> FastJobResult<bool> {
  let parts = ChatParticipant::list_participants_for_rooms(pool, &[room_id]).await?;
  Ok(parts.iter().any(|p| p.member_id == caller))
}

// Helper: update JobBudgetPlan.installments' status for a given workflow and seq
async fn _update_job_plan_step_status(
  pool: &mut app_108jobs_db::utils::DbPool<'_>,
  workflow_id: app_108jobs_db::newtypes::WorkflowId,
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
    .map_err(|_| FastJobErrorType::InvalidInstallmentsJson)?;

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

  let phases_json =
    serde_json::to_value(&steps).map_err(|_| FastJobErrorType::InvalidInstallmentsSerialization)?;
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
  data: Json<CreateInvoiceRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CreateInvoiceResponse>> {
  // Validate via TryFrom into a validated wrapper
  let validated: ValidCreateInvoiceRequest = data.into_inner().try_into()?;
  let workflow_id = validated.0.workflow_id;
  // Authz: caller must be the freelancer — a participant of the workflow's chat
  // room who is not the employer (post creator).
  let wf = Workflow::read(&mut context.pool(), workflow_id).await?;
  let post = Post::read(&mut context.pool(), wf.post_id).await?;
  if local_user_view.person.id == post.creator_id
    || !is_room_participant(
      &mut context.pool(),
      wf.room_id,
      local_user_view.local_user.id,
    )
    .await?
  {
    return Err(FastJobErrorType::NotFound.into());
  }
  // Create the invoice/billing record with detailed quotation fields
  let billing = WorkflowService::create_quotation(
    &mut context.pool(),
    local_user_view.local_user.id,
    validated,
  )
  .await?;
  let _ = Workflow::update_billing(&mut context.pool(), workflow_id, billing.id).await?;
  Ok(Json(CreateInvoiceResponse {
    billing_id: billing.id,
    issuer_id: local_user_view.local_user.id,
    recipient_id: billing.employer_id,
    post_id: billing.post_id,
    amount: billing.amount,
    status: billing.status,
    created_at: billing.created_at.to_rfc3339(),
    success: true,
  }))
}

pub async fn approve_quotation(
  data: Json<ApproveQuotationRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  let employer_id = local_user_view.local_user.id;

  // Validate
  let validated: ValidApproveQuotationRequest = data.into_inner().try_into()?;

  let form = validated.0;
  // Authz: caller must be the employer named on this billing.
  let billing = Billing::read(&mut context.pool(), form.billing_id).await?;
  require_role(
    WorkflowRole::Employer,
    employer_id,
    billing.employer_id,
    billing.freelancer_id,
  )?;
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
  data: Json<SubmitStartWorkRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  // Validate input
  let validated: ValidSubmitStartWorkRequest = data.into_inner().try_into()?;
  let form = validated.0;

  // Authz: only the freelancer may start work.
  let billing = billing_for_workflow(&mut context.pool(), form.workflow_id).await?;
  require_role(
    WorkflowRole::Freelancer,
    local_user_view.local_user.id,
    billing.employer_id,
    billing.freelancer_id,
  )?;

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
  data: Json<SubmitStartWorkRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  // Validate input
  let validated: ValidSubmitStartWorkRequest = data.into_inner().try_into()?;
  let form = validated.0;

  // Authz: only the freelancer may submit work.
  let billing = billing_for_workflow(&mut context.pool(), form.workflow_id).await?;
  require_role(
    WorkflowRole::Freelancer,
    local_user_view.local_user.id,
    billing.employer_id,
    billing.freelancer_id,
  )?;

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
  data: Json<ApproveWorkRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  // Validate input
  let validated: ValidApproveWorkRequest = data.into_inner().try_into()?;
  let form = validated.0;
  let workflow_id = form.workflow_id;
  ChatRoomId::try_from(form.room_id).map_err(|_| FastJobErrorType::InvalidRoomIdFormat)?;
  let billing_id = form.billing_id;
  // Authz: only the employer may approve work (this releases escrow to the freelancer).
  let billing = Billing::read(&mut context.pool(), billing_id).await?;
  require_role(
    WorkflowRole::Employer,
    local_user_view.local_user.id,
    billing.employer_id,
    billing.freelancer_id,
  )?;
  let coin_id = context.get_coin_id().await?;
  let platform_wallet_id = context.get_platform_wallet_id().await?;

  let wf = WorkflowService::load_work_submit(&mut context.pool(), workflow_id)
    .await?
    .approve_work_on(&mut context.pool(), coin_id, platform_wallet_id, billing_id)
    .await?;

  Ok(Json(WorkFlowOperationResponse {
    workflow_id: wf.data.workflow_id.into(),
    status: WorkFlowStatus::Completed,
    success: true,
  }))
}

pub async fn request_revision(
  data: Json<RequestRevisionRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  // Validate input
  let validated: ValidRequestRevisionRequest = data.into_inner().try_into()?;
  let form = validated.0;
  let workflow_id = form.workflow_id;

  // Authz: only the employer may request a revision.
  let billing = billing_for_workflow(&mut context.pool(), workflow_id).await?;
  require_role(
    WorkflowRole::Employer,
    local_user_view.local_user.id,
    billing.employer_id,
    billing.freelancer_id,
  )?;

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
  data: Json<UpdateBudgetPlanInstallmentsRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<UpdateBudgetPlanInstallmentsResponse>> {
  let post_id = data.post_id;
  // Authz: only the employer (post creator) may edit the budget plan.
  let post = Post::read(&mut context.pool(), post_id).await?;
  require_post_creator(local_user_view.person.id, post.creator_id)?;

  // Validate via TryFrom into a validated wrapper
  let validated: ValidUpdateBudgetPlanInstallmentsRequest = data.into_inner().try_into()?;

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
  data: Json<StartWorkflowRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  // Validate input
  let validated: ValidStartWorkflowRequest = data.into_inner().try_into()?;
  let form = validated.0;
  // Authz: only the employer (post creator) may start a workflow.
  let post = Post::read(&mut context.pool(), form.post_id).await?;
  require_post_creator(local_user_view.person.id, post.creator_id)?;
  let room_id =
    ChatRoomId::try_from(form.room_id).map_err(|_| FastJobErrorType::InvalidRoomIdFormat)?;
  let wf =
    WorkflowService::start_workflow(&mut context.pool(), form.post_id, form.seq_number, room_id)
      .await?;

  Ok(Json(WorkFlowOperationResponse {
    workflow_id: wf.id.into(),
    status: WorkFlowStatus::WaitForFreelancerQuotation,
    success: true,
  }))
}

pub async fn cancel_job(
  data: Json<CancelJobRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  // Validate input
  let validated: ValidCancelJobRequest = data.into_inner().try_into()?;
  let form = validated.0;

  // Authz: either party (employer or freelancer) may cancel.
  let wf = Workflow::read(&mut context.pool(), form.workflow_id).await?;
  match wf.billing_id {
    Some(billing_id) => {
      let billing = Billing::read(&mut context.pool(), billing_id).await?;
      require_any_party(
        local_user_view.local_user.id,
        billing.employer_id,
        billing.freelancer_id,
      )?;
    }
    None => {
      // Pre-billing: employer is the post creator; freelancer is a room member.
      let post = Post::read(&mut context.pool(), wf.post_id).await?;
      let is_employer = local_user_view.person.id == post.creator_id;
      let is_member = is_room_participant(
        &mut context.pool(),
        wf.room_id,
        local_user_view.local_user.id,
      )
      .await?;
      if !is_employer && !is_member {
        return Err(FastJobErrorType::NotFound.into());
      }
    }
  }

  // Perform cancellation (allowed for any non-finalized status)
  let _ =
    WorkflowService::cancel(&mut context.pool(), form.workflow_id, form.current_status).await?;
  // Refund any reserved/outstanding funds back to payer (idempotent inside service)
  let _ = WorkflowService::refund_on_cancel(&mut context.pool(), form.workflow_id).await;

  Ok(Json(WorkFlowOperationResponse {
    workflow_id: form.workflow_id.into(),
    status: WorkFlowStatus::Cancelled,
    success: true,
  }))
}

/// GET billing by room id where status is QuotePendingReview
pub async fn get_billing_by_room(
  query: Query<GetBillingByRoomQuery>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<Billing>> {
  let mut pool = context.pool();
  let room_id = ChatRoomId::try_from(query.room_id.clone())
    .map_err(|_| FastJobErrorType::InvalidRoomIdFormat)?;
  let billing_status = query
    .billing_status
    .unwrap_or(BillingStatus::QuotePendingReview);
  let bill_opt = Billing::get_by_room_and_status(&mut pool, room_id, billing_status).await?;

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
