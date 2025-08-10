use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::is_admin;
use lemmy_db_views_bank_account::api::{ListBankAccounts, ListBankAccountsResponse};
use lemmy_db_views_bank_account::BankAccountView;
use lemmy_db_views_local_user::LocalUserView;

use lemmy_utils::error::FastJobResult;

pub async fn list_bank_accounts(
  data: Query<ListBankAccounts>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListBankAccountsResponse>> {
  // Check if user is admin
  is_admin(&local_user_view)?;
  let verified = data.verify;
  let local_user_id = local_user_view.local_user.id;
  let bank_accounts = BankAccountView::list_by_user(&mut context.pool(), local_user_id, verified).await?;
  Ok(Json(ListBankAccountsResponse {
    bank_accounts,
  }))
}
