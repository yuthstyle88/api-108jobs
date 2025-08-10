use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::is_admin;
use lemmy_db_views_bank_account::api::{BankAccountResponse, ListBankAccounts, ListBankAccountsResponse};
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

  let response_accounts = bank_accounts
    .into_iter()
    .map(|view| BankAccountResponse {
      id: view.user_bank_account.id,
      user_id: view.user_bank_account.local_user_id,
      bank_id: view.bank.id,
      bank_name: view.bank.name,
      bank_country_id: view.bank.country_id,
      account_number: view.user_bank_account.account_number,
      account_name: view.user_bank_account.account_name,
      is_default: view.user_bank_account.is_default.unwrap_or(false),
      verification_image_path: view.user_bank_account.verification_image_path,
      created_at: view.user_bank_account.created_at.to_rfc3339(),
    })
    .collect();

  Ok(Json(ListBankAccountsResponse {
    bank_accounts: response_accounts,
  }))
}
