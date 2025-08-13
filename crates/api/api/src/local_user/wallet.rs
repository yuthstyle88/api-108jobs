use actix_web::web::{Data, Json};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::WalletId;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_wallet::api::{ApproveQuotation, ApproveWork, BillingOperationResponse, CreateInvoiceForm, CreateInvoiceResponse, DepositWallet, GetWalletResponse, SubmitWork, ValidCreateInvoice, WalletOperationResponse};
use lemmy_db_schema::source::wallet::WalletModel;
use lemmy_utils::error::FastJobResult;
use lemmy_workflow::WorkFlowService;

pub async fn get_wallet(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<GetWalletResponse>> {
  let user_id = local_user_view.local_user.id;

  let wallet = WalletModel::get_by_user(&mut context.pool(), user_id).await?;

  let response = GetWalletResponse {
    wallet_id: wallet.id,
    balance: wallet.balance_available,
    escrow_balance: wallet.balance_outstanding,
  };
  Ok(Json(response))
}

pub async fn deposit_wallet(
  data: Json<DepositWallet>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<WalletOperationResponse>> {
  let user_id = local_user_view.local_user.id;

  // Load user's wallet (must exist per NOT NULL constraint)
  let wallet = WalletModel::get_by_user(&mut context.pool(), user_id).await?;

  // Deposit funds
  let updated_wallet = WalletModel::deposit(&mut context.pool(), wallet.id, data.amount).await?;

  Ok(Json(WalletOperationResponse {
    wallet_id: updated_wallet.id,
    balance: updated_wallet.balance_available,
    escrow_balance: updated_wallet.balance_outstanding,
    transaction_amount: data.amount,
    success: true,
  }))
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
      return Err(lemmy_utils::error::FastJobErrorType::InvalidField(msg).into());
    }
  };
  let data = validated.0.clone();

  // Create the invoice/billing record with detailed quotation fields
  let billing = WorkFlowService::create_billing_from_quotation(
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
  let wallet_id: WalletId = local_user_view.local_user.wallet_id;
  // Approve the quotation and convert to order
  let updated_billing = WorkFlowService::approve_quotation(
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
  let updated_billing = WorkFlowService::submit_work(
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


pub async fn approve_work(
  data: Json<ApproveWork>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<BillingOperationResponse>> {
  let employer_id = local_user_view.local_user.id;

  // Approve work and release payment to worker
  let updated_billing = WorkFlowService::approve_work(
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

