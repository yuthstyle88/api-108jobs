use actix_web::{
  web::{Data, Json},
  HttpRequest,
};
use app_108jobs_api_utils::{
  claims::Claims,
  context::FastJobContext,
  utils::{check_local_user_deleted, check_local_user_valid},
};
use app_108jobs_core::error::FastJobResult;
use app_108jobs_db_schema::source::login_token::LoginToken;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::LoginResponse;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenRequest {
  /// The current (still-valid) session token to exchange for a fresh one.
  pub refresh_token: String,
}

/// POST /account/auth/refresh
///
/// Re-issues a session token so the client can keep a session alive without
/// re-entering credentials. There is no separate long-lived refresh-token
/// store in this system: the token presented here must itself still be valid
/// (signature + not expired + present in `login_token`). A new token with a
/// fresh TTL is minted and the presented one is rotated out.
pub async fn refresh_token(
  data: Json<RefreshTokenRequest>,
  req: HttpRequest,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<LoginResponse>> {
  let presented = data.refresh_token.clone();

  // Validates signature, expiry, and that the token is an active login token.
  let (user_id, _session) = Claims::validate(&presented, &context).await?;

  let local_user_view = LocalUserView::read(&mut context.pool(), user_id).await?;
  check_local_user_valid(&local_user_view)?;
  check_local_user_deleted(&local_user_view)?;

  let jwt = Claims::generate(
    local_user_view.local_user.id,
    local_user_view.local_user.email,
    local_user_view.local_user.interface_language,
    local_user_view.local_user.accepted_terms,
    local_user_view.local_user.admin,
    req,
    &context,
  )
  .await?;

  // Rotate: drop the presented token now that a replacement is issued.
  // Best-effort — the new token is already valid even if cleanup races.
  let _ = LoginToken::invalidate(&mut context.pool(), &presented).await;

  Ok(Json(LoginResponse {
    jwt: Some(jwt),
    verify_email_sent: false,
    registration_created: false,
    accepted_terms: false,
  }))
}
