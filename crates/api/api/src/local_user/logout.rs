use actix_web::web::Data;
use actix_web::{cookie::Cookie, HttpRequest, HttpResponse};
use lemmy_api_utils::{
  context::FastJobContext,
  utils::{read_auth_token, AUTH_COOKIE_NAME},
};
use lemmy_db_schema::source::login_token::LoginToken;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};

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
