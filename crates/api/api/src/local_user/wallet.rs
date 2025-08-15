use actix_web::web::{Data, Json};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::wallet::{TxKind, WalletModel, WalletTransactionInsertForm};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_wallet::api::{DepositWallet, GetWalletResponse, WalletOperationResponse};
use lemmy_utils::error::FastJobResult;

use uuid::Uuid;

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

  // Deposit funds: construct a wallet transaction insert form and call deposit
  let form = WalletTransactionInsertForm {
    wallet_id: wallet.id,
    reference_type: "user_deposit".to_string(),
    reference_id: 0,
    kind: TxKind::Deposit,
    amount: data.amount,
    description: "User deposit".to_string(),
    counter_user_id: None,
    idempotency_key: Uuid::new_v4().to_string(),
  };

  let updated_wallet = WalletModel::deposit(&mut context.pool(), &form).await?;

  Ok(Json(WalletOperationResponse {
    wallet_id: updated_wallet.id,
    balance: updated_wallet.balance_available,
    transaction_amount: data.amount,
    success: true,
  }))
}


