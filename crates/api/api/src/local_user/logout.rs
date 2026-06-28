use actix_web::{cookie::Cookie, web::Data, HttpRequest, HttpResponse};
use app_108jobs_api_utils::{
  context::FastJobContext,
  utils::{read_auth_token, AUTH_COOKIE_NAME},
};
use app_108jobs_core::error::{FastJobErrorType, FastJobResult};
use app_108jobs_db_schema::source::login_token::LoginToken;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::SuccessResponse;

pub async fn logout(
  req: HttpRequest,
  // require login
  _local_user_view: LocalUserView,
  context: Data<FastJobContext>,
) -> FastJobResult<HttpResponse> {
  let jwt = read_auth_token(&req)?.ok_or(FastJobErrorType::NotLoggedIn)?;
  LoginToken::invalidate(&mut context.pool(), &jwt).await?;

  let mut res = HttpResponse::Ok().json(SuccessResponse::default());
  let cookie = Cookie::new(AUTH_COOKIE_NAME, "");
  res.add_removal_cookie(&cookie)?;
  Ok(res)
}
