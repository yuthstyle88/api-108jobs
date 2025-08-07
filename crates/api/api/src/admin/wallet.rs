use actix_web::web::{Data, Json};
use lemmy_api_common::wallet::{AdminTopUpWallet, AdminWithdrawWallet, AdminWalletOperationResponse};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_views_wallet::WalletView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub async fn admin_top_up_wallet(
  data: Json<AdminTopUpWallet>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<AdminWalletOperationResponse>> {
  // Check if the user is admin
  if !local_user_view.local_user.admin {
    return Err(lemmy_utils::error::FastJobErrorType::NotAnAdmin.into());
  }

  let (updated_wallet, previous_balance) = WalletView::admin_top_up(
    &mut context.pool(),
    data.user_id,
    data.amount,
  ).await?;

  Ok(Json(AdminWalletOperationResponse {
    user_id: data.user_id,
    wallet_id: updated_wallet.id,
    previous_balance,
    new_balance: updated_wallet.balance,
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
  if !local_user_view.local_user.admin {
    return Err(lemmy_utils::error::FastJobErrorType::NotAnAdmin.into());
  }

  let (updated_wallet, previous_balance) = WalletView::admin_withdraw(
    &mut context.pool(),
    data.user_id,
    data.amount,
  ).await?;

  Ok(Json(AdminWalletOperationResponse {
    user_id: data.user_id,
    wallet_id: updated_wallet.id,
    previous_balance,
    new_balance: updated_wallet.balance,
    operation_amount: -data.amount, // Negative because it's a withdrawal
    reason: data.reason.clone(),
    success: true,
  }))
}