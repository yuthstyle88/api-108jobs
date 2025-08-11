use actix_web::web::{Data, Json};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::WalletId;
use lemmy_db_views_wallet::WalletView;
use lemmy_db_views_billing::BillingView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_wallet::api::{ApproveQuotation, ApproveWork, BillingOperationResponse, CreateInvoiceForm, CreateInvoiceResponse, DepositWallet, GetWalletResponse, RequestRevision, SubmitWork, UpdateWorkAfterRevision, WalletOperationResponse, ValidCreateInvoice};
use lemmy_utils::error::{ FastJobResult};

pub async fn get_wallet(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<GetWalletResponse>> {
  let user_id = local_user_view.local_user.id;

  let wallet_view = WalletView::read_by_user(&mut context.pool(), user_id).await?;


  let response =  GetWalletResponse {
      wallet_id: wallet_view.id,
      balance: wallet_view.balance,
      escrow_balance: wallet_view.escrow_balance,
    };
  Ok(Json(response))
}

pub async fn deposit_wallet(
  data: Json<DepositWallet>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<WalletOperationResponse>> {
  let user_id = local_user_view.local_user.id;

  // Create wallet if it doesn't exist
  let wallet_view = WalletView::read_by_user(&mut context.pool(), user_id).await?;

  let _ =  WalletView::create_for_user(&mut context.pool(), user_id).await?;
  let wallet_id = wallet_view.id;
  // Deposit funds
  let updated_wallet = WalletView::deposit_funds(&mut context.pool(), wallet_id, data.amount).await?;

  Ok(Json(WalletOperationResponse {
    wallet_id: updated_wallet.id,
    balance: updated_wallet.balance,
    escrow_balance: updated_wallet.escrow_balance,
    transaction_amount: data.amount,
    success: true,
  }))
}

// Escrow-based billing workflow handlers

pub async fn create_invoice(
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
  let billing = BillingView::create_invoice(
    &mut context.pool(),
    local_user_id,
    validated,
  ).await?;

  Ok(Json(CreateInvoiceResponse {
    billing_id: billing.id,
    issuer_id: local_user_id,
    recipient_id: data.employer_id,
    post_id: data.post_id,
    amount: data.price,
    status: "QuotationPending".to_string(),
    max_revisions: data.revise_times,
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
  let wallet_id = local_user_view.local_user.wallet_id.unwrap_or(WalletId(0));
  // Approve the quotation and convert to order
  let updated_billing = BillingView::approve_quotation(
    &mut context.pool(),
    data.billing_id,
    employer_id,
    wallet_id,
  ).await?;

  Ok(Json(BillingOperationResponse {
    billing_id: updated_billing.id,
    status: "PaidEscrow".to_string(),
    success: true,
  }))
}

pub async fn submit_work(
  data: Json<SubmitWork>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<BillingOperationResponse>> {
  let worker_id = local_user_view.local_user.id;

  // Submit the work as the worker
  let updated_billing = BillingView::submit_work(
    &mut context.pool(),
    data.billing_id,
    worker_id,
    data.work_description.clone(),
    data.deliverable_url.clone(),
  ).await?;

  Ok(Json(BillingOperationResponse {
    billing_id: updated_billing.id,
    status: "WorkSubmitted".to_string(),
    success: true,
  }))
}

pub async fn request_revision(
  data: Json<RequestRevision>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<BillingOperationResponse>> {
  let employer_id = local_user_view.local_user.id;

  // Request revision from worker
  let updated_billing = BillingView::request_revision(
    &mut context.pool(),
    data.billing_id,
    employer_id,
    data.revision_feedback.clone(),
  ).await?;

  Ok(Json(BillingOperationResponse {
    billing_id: updated_billing.id,
    status: "RequestChange".to_string(),
    success: true,
  }))
}

pub async fn approve_work(
  data: Json<ApproveWork>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<BillingOperationResponse>> {
  let employer_id = local_user_view.local_user.id;

  // Approve work and release payment to worker
  let updated_billing = BillingView::approve_work(
    &mut context.pool(),
    data.billing_id,
    employer_id,
  ).await?;

  Ok(Json(BillingOperationResponse {
    billing_id: updated_billing.id,
    status: "Completed".to_string(),
    success: true,
  }))
}

pub async fn update_work_after_revision(
  data: Json<UpdateWorkAfterRevision>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<BillingOperationResponse>> {
  let worker_id = local_user_view.local_user.id;

  // Update work after revision request as the worker
  let updated_billing = BillingView::update_work_after_revision(
    &mut context.pool(),
    data.billing_id,
    worker_id,
    data.updated_work_description.clone(),
    data.updated_deliverable_url.clone(),
  ).await?;

  Ok(Json(BillingOperationResponse {
    billing_id: updated_billing.id,
    status: "Updated".to_string(),
    success: true,
  }))
}