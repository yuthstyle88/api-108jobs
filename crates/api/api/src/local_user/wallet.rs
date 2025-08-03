use actix_web::web::{Data, Json};
use lemmy_api_common::wallet::{
  GetWalletResponse, DepositWallet, CreateInvoice, CreateInvoiceResponse, ApproveQuotation, 
  SubmitWork, ApproveWork, RequestRevision, WalletOperationResponse, BillingOperationResponse
};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_views_wallet::WalletView;
use lemmy_db_views_billing::BillingView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{ FastJobResult};

pub async fn get_wallet(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<GetWalletResponse>> {
  let user_id = local_user_view.local_user.id;

  let wallet_view = WalletView::read_by_user(&mut context.pool(), user_id).await?;

  let response = if let Some(wallet_view) = wallet_view {
    GetWalletResponse {
      wallet_id: Some(wallet_view.wallet.id),
      balance: wallet_view.wallet.balance,
      escrow_balance: wallet_view.wallet.escrow_balance,
    }
  } else {
    GetWalletResponse {
      wallet_id: None,
      balance: None,
      escrow_balance: None,
    }
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
  if wallet_view.is_none() {
    WalletView::create_for_user(&mut context.pool(), user_id).await?;
  }

  // Deposit funds
  let updated_wallet = WalletView::deposit_funds(&mut context.pool(), user_id, data.amount).await?;

  Ok(Json(WalletOperationResponse {
    wallet_id: updated_wallet.id,
    balance: updated_wallet.balance.unwrap_or(0.0),
    escrow_balance: updated_wallet.escrow_balance.unwrap_or(0.0),
    transaction_amount: data.amount,
    success: true,
  }))
}

// Escrow-based billing workflow handlers

pub async fn create_invoice(
  data: Json<CreateInvoice>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CreateInvoiceResponse>> {
  let freelancer_id = local_user_view.local_user.id;
  
  // Validate input
  if data.price <= 0.0 {
    return Err(lemmy_utils::error::FastJobErrorType::InvalidField("Price must be positive".to_string()).into());
  }
  
  if data.revise_times < 0 {
    return Err(lemmy_utils::error::FastJobErrorType::InvalidField("Revise times cannot be negative".to_string()).into());
  }

  if data.working_days <= 0 {
    return Err(lemmy_utils::error::FastJobErrorType::InvalidField("Working days must be positive".to_string()).into());
  }

  // Create the invoice/billing record with detailed quotation fields
  let billing = BillingView::create_invoice(
    &mut context.pool(),
    freelancer_id,
    data.employer_id,
    data.post_id,
    data.comment_id,
    data.price,
    data.proposal.clone(),
    data.name.clone(),
    data.job_description.clone(),
    data.work_steps.clone(),
    data.revise_times,
    data.revise_description.clone(),
    data.working_days,
    data.deliverables.clone(),
    data.note.clone(),
    data.starting_day.clone(),
    data.delivery_day.clone(),
  ).await?;

  Ok(Json(CreateInvoiceResponse {
    billing_id: billing.id,
    freelancer_id,
    employer_id: data.employer_id,
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

  // Approve the quotation and convert to order
  let updated_billing = BillingView::approve_quotation(
    &mut context.pool(),
    data.billing_id,
    employer_id,
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
  let freelancer_id = local_user_view.local_user.id;

  // Submit the work
  let updated_billing = BillingView::submit_work(
    &mut context.pool(),
    data.billing_id,
    freelancer_id,
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

  // Request revision from freelancer
  let updated_billing = BillingView::request_revision(
    &mut context.pool(),
    data.billing_id,
    employer_id,
    data.revision_feedback.clone(),
  ).await?;

  Ok(Json(BillingOperationResponse {
    billing_id: updated_billing.id,
    status: "PaidEscrow".to_string(),
    success: true,
  }))
}

pub async fn approve_work(
  data: Json<ApproveWork>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<BillingOperationResponse>> {
  let employer_id = local_user_view.local_user.id;

  // Approve work and release payment to freelancer
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