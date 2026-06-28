use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::{context::FastJobContext, utils::list_withdraw_requests_inner};
use app_108jobs_core::error::{FastJobErrorType, FastJobResult};
use app_108jobs_db::{
  newtypes::WithdrawRequestId,
  source::{
    currency::Currency,
    user_bank_account::BankAccount,
    wallet::WalletModel,
    withdraw_request::{WithdrawRequest, WithdrawRequestInsertForm},
  },
  traits::Crud,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_db_views_wallet::{
  ListWithdrawRequestQuery,
  ListWithdrawRequestResponse,
  SubmitWithdrawRequest,
  ValidSubmitWithdrawRequest,
};

pub async fn submit_withdraw(
  data: Json<SubmitWithdrawRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let validated: ValidSubmitWithdrawRequest = data.into_inner().try_into()?;

  // Verify wallet belongs to the authenticated user
  let wallet = WalletModel::get_by_user(&mut context.pool(), local_user_view.local_user.id).await?;
  if wallet.id != validated.0.wallet_id {
    return Err(FastJobErrorType::NotFound.into());
  }

  // Verify bank account belongs to the authenticated user
  let bank_account = BankAccount::read(&mut context.pool(), validated.0.bank_account_id).await?;
  if bank_account.local_user_id != local_user_view.local_user.id {
    return Err(FastJobErrorType::NotFound.into());
  }

  // Get the currency to calculate the conversion rate
  let currency = Currency::read(&mut context.pool(), validated.0.currency_id).await?;

  // Calculate amount in the selected currency
  let amount_currency = currency.coins_to_currency(validated.0.amount.0);

  let insert_form = WithdrawRequestInsertForm {
    local_user_id: local_user_view.local_user.id,
    wallet_id: validated.0.wallet_id,
    user_bank_account_id: validated.0.bank_account_id,
    amount: validated.0.amount,
    currency_id: validated.0.currency_id,
    amount_currency,
    conversion_rate_used: currency.coin_to_currency_rate,
    reason: Some(validated.0.reason),
  };

  let _created = WithdrawRequest::create(&mut context.pool(), &insert_form).await?;

  Ok(Json(SuccessResponse::default()))
}

pub async fn list_withdraw_requests(
  query: Query<ListWithdrawRequestQuery>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListWithdrawRequestResponse>> {
  let res = list_withdraw_requests_inner(
    &mut context.pool(),
    Some(local_user_view.local_user.id),
    query.into_inner(),
  )
  .await?;
  Ok(Json(res))
}

pub async fn retract_withdraw(
  path: actix_web::web::Path<WithdrawRequestId>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let id = path.into_inner();
  WithdrawRequest::cancel_by_user(&mut context.pool(), id, local_user_view.local_user.id).await?;
  Ok(Json(SuccessResponse::default()))
}
