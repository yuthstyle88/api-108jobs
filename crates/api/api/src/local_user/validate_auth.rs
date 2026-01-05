use actix_web::{
  web::{Data, Json},
  HttpRequest,
};
use app_108jobs_api_utils::{
  context::FastJobContext,
  utils::{local_user_view_from_jwt, read_auth_token},
};
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};

/// Returns an error message if the auth token is invalid for any reason. Necessary because other
/// endpoints silently treat any call with invalid auth as unauthenticated.
pub async fn validate_auth(
  req: HttpRequest,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<SuccessResponse>> {
  let jwt = read_auth_token(&req)?;
  if let Some(jwt) = jwt {
    local_user_view_from_jwt(&jwt, &context).await?;
  } else {
    Err(FastJobErrorType::NotLoggedIn)?;
  }
  Ok(Json(SuccessResponse::default()))
}
