use actix_web::web::{Data, Json};
use lemmy_api_common::bank_account::{
  BankAccountOperationResponse, BankResponse, CreateUserBankAccount,
  DeleteBankAccount, ListBanksResponse, ListUserBankAccountsResponse,
  SetDefaultBankAccount, UserBankAccountResponse
};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::bank::Bank;
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_address::AddressView;
use lemmy_db_views_bank_account::{BankView, UserBankAccountView};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub async fn create_bank_account(
  data: Json<CreateUserBankAccount>,
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

  let user_bank_account = UserBankAccountView::create(
    &mut context.pool(),
    user_id,
    data.bank_id,
    data.account_number.clone(),
    data.account_name.clone(),
    data.verification_image.clone(),
  ).await?;

  Ok(Json(BankAccountOperationResponse {
    bank_account_id: user_bank_account.id,
    success: true,
  }))
}

pub async fn list_user_bank_accounts(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListUserBankAccountsResponse>> {
  let user_id = local_user_view.local_user.id;

  let bank_accounts = UserBankAccountView::list_by_user(&mut context.pool(), user_id).await?;

  let response_accounts = bank_accounts
    .into_iter()
    .map(|view| UserBankAccountResponse {
      id: view.user_bank_account.id,
      bank_id: view.bank.id,
      bank_name: view.bank.name,
      bank_country_id: view.bank.country_id,
      account_number: view.user_bank_account.account_number,
      account_name: view.user_bank_account.account_name,
      is_default: view.user_bank_account.is_default.unwrap_or(false),
      is_verified: view.user_bank_account.is_verified,
      created_at: view.user_bank_account.created_at.to_rfc3339(),
    })
    .collect();

  Ok(Json(ListUserBankAccountsResponse {
    bank_accounts: response_accounts,
  }))
}

pub async fn set_default_bank_account(
  data: Json<SetDefaultBankAccount>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<BankAccountOperationResponse>> {
  let user_id = local_user_view.local_user.id;

  let _updated_account = UserBankAccountView::set_default(
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

  let _result = UserBankAccountView::delete(
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
) -> FastJobResult<Json<ListBanksResponse>> {
  let address_id = local_user_view.person.address_id;

  let address_view = AddressView::find_by_id(&mut context.pool(), address_id).await?;
  // Get active banks then filter by user's country
  let banks = BankView::list(&mut context.pool()).await?;

  let response_banks = banks
    .into_iter()
    .filter(|bank| bank.country_id == address_view.address.country_id)
    .map(|bank| BankResponse {
      id: bank.id,
      name: bank.name,
      country_id: bank.country_id,
      bank_code: bank.bank_code,
      swift_code: bank.swift_code,
    })
    .collect();

  Ok(Json(ListBanksResponse {
    banks: response_banks,
  }))
}