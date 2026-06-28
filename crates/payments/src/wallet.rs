use actix_web::web::{Data, Json};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_core::error::FastJobResult;
use app_108jobs_db::source::wallet::WalletModel;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_wallet::api::GetWalletResponse;

pub async fn get_wallet(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<GetWalletResponse>> {
  let local_user_id = local_user_view.local_user.id;

  let wallet = WalletModel::get_by_user(&mut context.pool(), local_user_id).await?;

  let response = GetWalletResponse {
    wallet_id: wallet.id,
    balance: wallet.balance_available,
    escrow_balance: wallet.balance_outstanding,
  };
  Ok(Json(response))
}
