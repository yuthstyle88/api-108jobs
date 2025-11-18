use actix_web::web::{Data, Json, Query};
use chrono::Utc;
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::is_admin;
use lemmy_db_schema::source::user_bank_account::{BankAccount, UserBankAccountUpdateForm};
use lemmy_db_schema::traits::{Crud, PaginationCursorBuilder};
use lemmy_db_views_bank_account::api::{
  ListBankAccountQuery, ListBankAccountsResponse, VerifyBankAccount,
};
use lemmy_db_views_bank_account::BankAccountView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::FastJobResult;

pub async fn admin_list_bank_accounts(
  data: Query<ListBankAccountQuery>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListBankAccountsResponse>> {
  // Check if user is admin
  is_admin(&local_user_view)?;
  let data = data.into_inner();

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(BankAccountView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let items = BankAccountView::list(&mut context.pool(), None, cursor_data, data).await?;
  let next_page = items.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = items.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListBankAccountsResponse {
    bank_accounts: items,
    next_page,
    prev_page,
  }))
}

pub async fn admin_verify_bank_account(
  data: Json<VerifyBankAccount>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let data = data.into_inner();
  // Check if user is admin
  is_admin(&local_user_view)?;
  let update_form = UserBankAccountUpdateForm {
    bank_id: None,
    account_number: None,
    account_name: None,
    is_default: None,
    is_verified: Some(true),
    updated_at: Some(Some(Utc::now())),
    verification_image_path: None,
  };
  let _result =
    BankAccount::update(&mut context.pool(), data.bank_account_id, &update_form).await?;
  Ok(Json(SuccessResponse::default()))
}
