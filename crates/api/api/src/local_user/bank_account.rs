use actix_web::web::{Data, Json, Query};
use app_108jobs_api_common::bank_account::BankAccountOperationResponse;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_api_utils::utils::ensure_bank_account_unique_for_user;
use app_108jobs_db_schema::source::bank::{Bank, BanksResponse};
use app_108jobs_db_schema::source::user_bank_account::{
  BankAccount, UserBankAccountInsertForm, UserBankAccountUpdateForm,
};
use app_108jobs_db_schema::traits::Crud;
use app_108jobs_db_views_bank_account::api::{
  BankAccountForm, CreateBankAccount, DeleteBankAccount, GetBankAccounts, ListBankAccountsResponse,
  SetDefaultBankAccount, UpdateBankAccount,
};
use app_108jobs_db_views_bank_account::BankAccountView;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};

pub async fn create_bank_account(
  data: Json<BankAccountForm>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<BankAccountOperationResponse>> {
  let local_user_id = local_user_view.local_user.id;
  let data: CreateBankAccount = data.into_inner().try_into()?;

  let count = BankAccount::count_for_user(&mut context.pool(), &local_user_id).await?;
  if count >= 3 {
    return Err(FastJobErrorType::ReachedMax3BankAccounts.into());
  }

  // Verify bank belongs to user's country
  let bank = Bank::read(&mut context.pool(), data.bank_id)
    .await
    .map_err(|_| FastJobErrorType::InvalidField("Bank not found".to_string()))?;

  ensure_bank_account_unique_for_user(
    &mut context.pool(),
    &local_user_id,
    &data.bank_id,
    &data.account_number,
    None,
  )
  .await?;

  let verification_image = data.verification_image.clone();
  let bank_id = data.bank_id;
  let account_number = data.account_number.clone();
  let account_name = data.account_name.clone();
  let mut form = UserBankAccountInsertForm {
    local_user_id: local_user_id.clone(),
    bank_id,
    account_number,
    account_name,
    verification_image_path: verification_image.map(|_| {
      format!(
        "verification_images/user_{}/bank_account_{}.jpg",
        local_user_id.0, bank.id.0
      )
    }),
  };

  let user_bank_account = BankAccount::create(&mut context.pool(), &mut form).await?;

  let bank_account_view = BankAccountView::read(&mut context.pool(), user_bank_account.id).await?;

  Ok(Json(BankAccountOperationResponse {
    bank_account: bank_account_view,
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
  let bank_accounts =
    BankAccountView::list_by_user(&mut context.pool(), Some(local_user_id), verify).await?;

  Ok(Json(ListBankAccountsResponse {
    bank_accounts,
    next_page: None,
    prev_page: None,
  }))
}

pub async fn set_default_bank_account(
  data: Json<SetDefaultBankAccount>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let local_user_id = local_user_view.local_user.id;

  let _updated_account =
    BankAccount::set_default(&mut context.pool(), local_user_id, data.bank_account_id).await?;

  Ok(Json(SuccessResponse::default()))
}

pub async fn update_bank_account(
  data: Json<UpdateBankAccount>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<BankAccountOperationResponse>> {
  let local_user_id = local_user_view.local_user.id;
  let data = data.into_inner();

  let before_bank_account = BankAccount::read(&mut context.pool(), data.bank_account_id).await?;

  let bank_id = data
    .bank_id
    .as_ref()
    .cloned()
    .unwrap_or(before_bank_account.bank_id.clone());

  let account_number = data
    .account_number
    .as_ref()
    .cloned()
    .unwrap_or(before_bank_account.account_number.clone());

  ensure_bank_account_unique_for_user(
    &mut context.pool(),
    &local_user_id,
    &bank_id,
    &account_number,
    None,
  )
  .await?;

  let update_form = UserBankAccountUpdateForm {
    bank_id: Some(bank_id),
    account_number: Some(account_number),
    account_name: data.account_name,
    is_default: Some(false),
    is_verified: Some(false),
    updated_at: Some(Some(chrono::Utc::now())),
    verification_image_path: None,
  };

  let updated =
    BankAccount::update(&mut context.pool(), data.bank_account_id, &update_form).await?;

  let bank_account_view = BankAccountView::read(&mut context.pool(), updated.id).await?;

  Ok(Json(BankAccountOperationResponse {
    bank_account: bank_account_view,
    success: true,
  }))
}

pub async fn delete_bank_account(
  data: Json<DeleteBankAccount>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let local_user_id = local_user_view.local_user.id;

  let bank_account = BankAccount::read(&mut context.pool(), data.bank_account_id).await?;

  // Ownership check
  if bank_account.local_user_id != local_user_id {
    return Err(FastJobErrorType::UnauthorizedAccess.into());
  }

  // prevent deleting default bank account
  if bank_account.is_default {
    return Err(FastJobErrorType::CannotDeleteDefaultBankAccount.into());
  }

  let _deleted = BankAccount::delete(&mut context.pool(), data.bank_account_id).await?;

  Ok(Json(SuccessResponse::default()))
}

pub async fn list_banks(
  context: Data<FastJobContext>,
  _local_user_view: LocalUserView,
) -> FastJobResult<Json<BanksResponse>> {
  let bank_accounts = Bank::query_with_order_by(&mut context.pool(), None).await?;

  Ok(Json(bank_accounts))
}
