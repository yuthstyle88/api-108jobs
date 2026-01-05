use actix_web::web::{Data, Json, Query};
use chrono::Utc;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_api_utils::utils::{is_admin, list_top_up_requests_inner, list_withdraw_requests_inner};
use app_108jobs_db_schema::newtypes::CoinId;
use app_108jobs_db_schema::source::top_up_request::{TopUpRequest, TopUpRequestUpdateForm};
use app_108jobs_db_schema::source::wallet::{TxKind, WalletModel, WalletTransactionInsertForm};
use app_108jobs_db_schema::source::withdraw_request::{WithdrawRequest, WithdrawRequestUpdateForm};
use app_108jobs_db_schema::traits::Crud;
use app_108jobs_db_schema_file::enums::WithdrawStatus;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_db_views_wallet::api::{
  AdminTopUpWallet, AdminWalletOperationResponse, AdminWithdrawWallet, ListTopUpRequestQuery,
  ListTopUpRequestResponse, ListWithdrawRequestQuery, ListWithdrawRequestResponse,
  RejectWithdrawalRequest,
};
use app_108jobs_utils::error::FastJobResult;
use uuid::Uuid;

pub async fn admin_list_top_up_requests(
  query: Query<ListTopUpRequestQuery>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListTopUpRequestResponse>> {
  // Ensure admin access
  is_admin(&local_user_view)?;
  let res = list_top_up_requests_inner(&mut context.pool(), None, query.into_inner()).await?;

  Ok(Json(res))
}

pub async fn admin_top_up_wallet(
  data: Json<AdminTopUpWallet>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<AdminWalletOperationResponse>> {
  // Check if the user is admin
  is_admin(&local_user_view)?;

  let site_config = context.site_config().get().await?;
  let coin_id = site_config
    .site_view
    .local_site
    .coin_id
    .unwrap_or(CoinId(1));
  let platform_wallet_id = site_config
    .admins
    .first()
    .expect("At least one admin must exist to perform admin wallet operations")
    .person
    .wallet_id;

  let target_wallet = WalletModel::get_by_user(&mut context.pool(), data.target_user_id).await?;

  let form = WalletTransactionInsertForm {
    wallet_id: target_wallet.id,
    reference_type: "admin_top_up".to_string(),
    reference_id: 0,
    kind: TxKind::Deposit,
    amount: data.amount,
    description: data.reason.clone(),
    counter_user_id: Some(local_user_view.local_user.id),
    idempotency_key: Uuid::new_v4().to_string(),
  };

  let wallet =
    WalletModel::create_transaction(&mut context.pool(), &form, coin_id, platform_wallet_id)
      .await?;

  let wallet_topup_update_form = TopUpRequestUpdateForm {
    status: None,
    updated_at: Some(Utc::now()),
    paid_at: None,
    transferred: Some(true),
  };
  let _updated = TopUpRequest::update_by_qr_id(
    &mut context.pool(),
    data.qr_id.clone(),
    &wallet_topup_update_form,
  )
  .await?;

  Ok(Json(AdminWalletOperationResponse {
    wallet_id: wallet.id,
    new_balance: wallet.balance_total,
    operation_amount: data.amount,
    reason: data.reason.clone(),
    success: true,
  }))
}

pub async fn admin_withdraw_wallet(
  data: Json<AdminWithdrawWallet>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<AdminWalletOperationResponse>> {
  // Check if user is admin
  is_admin(&local_user_view)?;

  // Fetch site config once to avoid redundant await calls and clones
  let site_config = context.site_config().get().await?;
  let coin_id = site_config
    .site_view
    .local_site
    .coin_id
    .unwrap_or(CoinId(1));
  let platform_wallet_id = site_config
    .admins
    .first()
    .expect("At least one admin must exist to perform admin wallet operations")
    .person
    .wallet_id;

  let target_wallet = WalletModel::get_by_user(&mut context.pool(), data.target_user_id).await?;

  let form = WalletTransactionInsertForm {
    wallet_id: target_wallet.id,
    reference_type: "admin_withdraw".to_string(),
    reference_id: 0,
    kind: TxKind::Withdraw,
    amount: data.amount,
    description: data.reason.clone(),
    counter_user_id: Some(local_user_view.local_user.id),
    idempotency_key: Uuid::new_v4().to_string(),
  };

  let wallet =
    WalletModel::create_transaction(&mut context.pool(), &form, coin_id, platform_wallet_id)
      .await?;

  let withdrawal_update_form = WithdrawRequestUpdateForm {
    status: Some(WithdrawStatus::Completed),
    updated_at: Some(Utc::now()),
    reason: Some(Some(data.reason.clone())),
  };

  let _updated = WithdrawRequest::update(
    &mut context.pool(),
    data.withdrawal_id,
    &withdrawal_update_form,
  )
  .await?;

  Ok(Json(AdminWalletOperationResponse {
    wallet_id: wallet.id,
    new_balance: wallet.balance_total,
    operation_amount: -data.amount,
    reason: data.reason.clone(),
    success: true,
  }))
}

pub async fn admin_list_withdraw_requests(
  query: Query<ListWithdrawRequestQuery>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListWithdrawRequestResponse>> {
  is_admin(&local_user_view)?;
  let res = list_withdraw_requests_inner(&mut context.pool(), None, query.into_inner()).await?;

  Ok(Json(res))
}

pub async fn admin_reject_withdraw_request(
  data: Json<RejectWithdrawalRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  is_admin(&local_user_view)?;

  let update_form = WithdrawRequestUpdateForm {
    status: Some(WithdrawStatus::Rejected),
    updated_at: Some(Utc::now()),
    reason: Some(Some(data.reason.clone())),
  };

  let _updated =
    WithdrawRequest::update(&mut context.pool(), data.withdrawal_id, &update_form).await?;

  Ok(Json(SuccessResponse::default()))
}
