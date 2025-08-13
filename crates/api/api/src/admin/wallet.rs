use actix_web::web::{Data, Json};
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::is_admin;
use lemmy_db_schema::source::wallet::{WalletModel, WalletTransactionInsertForm, TxKind};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_wallet::api::{AdminTopUpWallet, AdminWalletOperationResponse, AdminWithdrawWallet};
use lemmy_utils::error::FastJobResult;
use uuid::Uuid;
use lemmy_utils::utils::validation::round_to_2_decimals;

pub async fn admin_top_up_wallet(
  data: Json<AdminTopUpWallet>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<AdminWalletOperationResponse>> {
  // Check if the user is admin
  is_admin(&local_user_view)?;

  let form = WalletTransactionInsertForm {
    wallet_id: data.wallet_id,
    reference_type: "admin_top_up".to_string(),
    reference_id: 0,
    kind: TxKind::Deposit,
    amount: round_to_2_decimals(data.amount),
    description: data.reason.clone(),
    counter_user_id: None,
    idempotency_key: Uuid::new_v4().to_string(),
  };

  let wallet = WalletModel::create_transaction(&mut context.pool(), &form).await?;

  Ok(Json(AdminWalletOperationResponse {
    wallet_id: data.wallet_id,
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

  let form = WalletTransactionInsertForm {
    wallet_id: data.wallet_id,
    reference_type: "admin_withdraw".to_string(),
    reference_id: 0,
    kind: TxKind::Withdraw,
    amount: round_to_2_decimals(data.amount),
    description: data.reason.clone(),
    counter_user_id: None,
    idempotency_key: Uuid::new_v4().to_string(),
  };

  let wallet = WalletModel::create_transaction(&mut context.pool(), &form).await?;

  Ok(Json(AdminWalletOperationResponse {
    wallet_id: wallet.id,
    new_balance: wallet.balance_total,
    operation_amount: -data.amount, // Negative because it's a withdrawal
    reason: data.reason.clone(),
    success: true,
  }))
}