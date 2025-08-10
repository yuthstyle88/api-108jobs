use actix_web::web::{Data, Json};
use lemmy_api_common::bank_account::{
  BankAccountOperationResponse,
};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_views_bank_account::api::{ListUnverifiedBankAccountsResponse, UnverifiedBankAccountResponse, VerifyBankAccount};
use lemmy_db_views_bank_account::BankAccountView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};

pub async fn list_unverified_bank_accounts(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListUnverifiedBankAccountsResponse>> {
  // Check if user is admin
  if !local_user_view.local_user.admin {
    return Err(FastJobErrorType::NotAnAdmin)?;
  }

  let local_user_id = local_user_view.local_user.id;
  let verified = Some(false);
  let bank_accounts = BankAccountView::list_by_user(&mut context.pool(), local_user_id, verified).await?;

  let response_accounts = bank_accounts
    .into_iter()
    .map(|view| UnverifiedBankAccountResponse {
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

  Ok(Json(ListUnverifiedBankAccountsResponse {
    bank_accounts: response_accounts,
  }))
}

pub async fn verify_bank_account(
  data: Json<VerifyBankAccount>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<BankAccountOperationResponse>> {
  // Check if the user is admin
  if !local_user_view.local_user.admin {
    return Err(FastJobErrorType::NotAnAdmin)?;
  }

  let _updated_account = BankAccountView::update_verify(
    &mut context.pool(),
    data.bank_account_id,
    data.verified,
  )
  .await?;

  // TODO: Store admin_notes if provided

  Ok(Json(BankAccountOperationResponse {
    bank_account_id: data.bank_account_id,
    success: true,
  }))
}
