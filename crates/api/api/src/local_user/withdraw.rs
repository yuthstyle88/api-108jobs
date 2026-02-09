use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_api_utils::utils::list_withdraw_requests_inner;
use app_108jobs_db_schema::source::currency::Currency;
use app_108jobs_db_schema::source::withdraw_request::{WithdrawRequest, WithdrawRequestInsertForm};
use app_108jobs_db_schema::traits::Crud;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_db_views_wallet::api::{
  ListWithdrawRequestQuery, ListWithdrawRequestResponse, SubmitWithdrawRequest,
  ValidWithdrawRequest,
};
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};

pub async fn submit_withdraw(
  data: Json<SubmitWithdrawRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let validated: ValidWithdrawRequest = match data.into_inner().try_into() {
    Ok(v) => v,
    Err(msg) => {
      return Err(FastJobErrorType::InvalidField(msg.to_string()).into());
    }
  };

  // Get the currency to calculate the conversion rate
  let currency = Currency::read(&mut context.pool(), validated.0.currency_id).await?;

  // Calculate amount in the selected currency
  let amount_currency = currency.coins_to_currency(validated.0.amount .0);

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
