use actix_web::web::{Data, Json, Query};
use lemmy_api_common::bank_account::BankAccountOperationResponse;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::bank::{Bank, BanksResponse};
use lemmy_db_schema::source::user_bank_account::UserBankAccountInsertForm;
use lemmy_db_schema::traits::Crud;

use lemmy_db_views_bank_account::api::{BankAccountForm, CreateBankAccount, DeleteBankAccount, GetBankAccounts, ListBankAccountsResponse, SetDefaultBankAccount};
use lemmy_db_views_bank_account::BankAccountView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub async fn create_bank_account(
  data: Json<BankAccountForm>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<BankAccountOperationResponse>> {
  let user_id = local_user_view.local_user.id;
  let data: CreateBankAccount = data.into_inner().try_into()?;
  // Load the user's address to determine the allowed country

  // Verify bank belongs to user's country
  let bank = Bank::read(&mut context.pool(), data.bank_id)
    .await
    .map_err(|_| lemmy_utils::error::FastJobErrorType::InvalidField("Bank not found".to_string()))?;

  let verification_image = data.verification_image.clone();
  let bank_id = data.bank_id;
  let account_number = data.account_number.clone();
  let account_name = data.account_name.clone();
  let mut form = UserBankAccountInsertForm {
    local_user_id: user_id,
    bank_id,
    account_number,
    account_name,
    is_default: None,
    verification_image_path: verification_image.map(|_| format!(
      "verification_images/user_{}/bank_account_{}.jpg",
      user_id.0, bank.id.0
    )),
  };

  let user_bank_account = BankAccountView::create(
    &mut context.pool(),
    &mut form
  ).await?;

  Ok(Json(BankAccountOperationResponse {
    bank_account_id: user_bank_account.id,
    success: true,
  }))
}

pub async fn list_user_bank_accounts(
  data: Query<GetBankAccounts>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListBankAccountsResponse>> {
  let local_user_id = local_user_view.local_user.id;
  let verify = data.is_verified;
  let bank_accounts = BankAccountView::list_by_user(&mut context.pool(), Some(local_user_id), verify).await?;

  Ok(Json(ListBankAccountsResponse {
    bank_accounts,
  }))
}

pub async fn set_default_bank_account(
  data: Json<SetDefaultBankAccount>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<BankAccountOperationResponse>> {
  let local_user_id = local_user_view.local_user.id;

  let _updated_account = BankAccountView::set_default(
    &mut context.pool(),
    local_user_id,
    data.bank_account_id,
  ).await?;

  Ok(Json(BankAccountOperationResponse {
    bank_account_id: data.bank_account_id,
    success: true,
  }))
}

pub async fn delete_bank_account(
  data: Json<DeleteBankAccount>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<BankAccountOperationResponse>> {
  let local_user_id = local_user_view.local_user.id;

  let _result = BankAccountView::delete(
    &mut context.pool(),
    local_user_id,
    data.bank_account_id,
  ).await?;

  Ok(Json(BankAccountOperationResponse {
    bank_account_id: data.bank_account_id,
    success: true,
  }))
}

pub async fn list_banks(
  context: Data<FastJobContext>,

) -> FastJobResult<Json<BanksResponse>> {

  let bank_accounts = Bank::query_with_order_by( &mut context.pool() ,None).await?;

  Ok(Json(bank_accounts))
}