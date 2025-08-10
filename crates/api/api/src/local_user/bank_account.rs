use actix_web::web::{Data, Json};
use lemmy_api_common::bank_account::{
  BankAccountOperationResponse
};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::bank::Bank;
use lemmy_db_schema::source::user_bank_account::UserBankAccountInsertForm;
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_address::AddressView;
use lemmy_db_views_bank_account::{BankAccountView, ListBankAccountsResponse};
use lemmy_db_views_bank_account::api::{CreateBankAccount, DeleteBankAccount, SetDefaultBankAccount};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub async fn create_bank_account(
  data: Json<CreateBankAccount>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<BankAccountOperationResponse>> {
  let user_id = local_user_view.local_user.id;
  let user_address_id = local_user_view.person.address_id;

  // Validate account number format
  if data.account_number.trim().is_empty() || data.account_number.len() > 50 {
    return Err(lemmy_utils::error::FastJobErrorType::InvalidField("Invalid account number".to_string()))?;
  }

  // Validate account name
  if data.account_name.trim().is_empty() || data.account_name.len() > 255 {
    return Err(lemmy_utils::error::FastJobErrorType::InvalidField("Invalid account name".to_string()))?;
  }

  // Load user's address to determine allowed country
  let address_view = AddressView::find_by_id(&mut context.pool(), user_address_id).await?;

  // Verify bank belongs to user's country
  let bank = Bank::read(&mut context.pool(), data.bank_id)
    .await
    .map_err(|_| lemmy_utils::error::FastJobErrorType::InvalidField("Bank not found".to_string()))?;
  
  if bank.country_id != address_view.address.country_id {
    return Err(lemmy_utils::error::FastJobErrorType::InvalidField(
      format!(
        "Bank {} is not available in your region ({})",
        bank.name, address_view.address.country_id
      )
    ))?;
  }
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
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListBankAccountsResponse>> {
  let user_id = local_user_view.local_user.id;

  let bank_accounts = BankAccountView::list_by_user(&mut context.pool(), user_id, Some(true)).await?;

  Ok(Json(ListBankAccountsResponse {
    bank_accounts,
  }))
}

pub async fn set_default_bank_account(
  data: Json<SetDefaultBankAccount>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<BankAccountOperationResponse>> {
  let user_id = local_user_view.local_user.id;

  let _updated_account = BankAccountView::set_default(
    &mut context.pool(),
    user_id,
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
  let user_id = local_user_view.local_user.id;

  let _result = BankAccountView::delete(
    &mut context.pool(),
    user_id,
    data.bank_account_id,
  ).await?;

  Ok(Json(BankAccountOperationResponse {
    bank_account_id: data.bank_account_id,
    success: true,
  }))
}

pub async fn list_banks(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListBankAccountsResponse>> {
   let local_user_id = Some(local_user_view.local_user.id);
  let order_by = Some("bank_name_desc".to_string());
  // Get active banks then filter by user's country
  let bank_accounts = BankAccountView::query_with_filters(&mut context.pool(), local_user_id, None, order_by).await?;

  Ok(Json(ListBankAccountsResponse {
    bank_accounts
  }))
}