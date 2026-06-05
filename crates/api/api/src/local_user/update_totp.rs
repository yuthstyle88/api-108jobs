use crate::check_totp_2fa_valid;
use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::source::local_user::{LocalUser, LocalUserUpdateForm};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::{UpdateTotpRequest, UpdateTotpResponse};
use app_108jobs_utils::error::FastJobResult;

/// Enable or disable two-factor-authentication. The current setting is determined from
/// [LocalUser.totp_2fa_enabled].
///
/// To enable, you need to first call [generate_totp_secret] and then pass a valid token to this
/// function.
///
/// Disabling is only possible if 2FA was previously enabled. Again it is necessary to pass a valid
/// token.
pub async fn update_totp(
  data: Json<UpdateTotpRequest>,
  local_user_view: LocalUserView,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<UpdateTotpResponse>> {
  check_totp_2fa_valid(
    &local_user_view,
    &Some(data.totp_token.clone()),
    &context.settings().hostname,
  )?;

  // toggle the 2fa setting
  let local_user_form = LocalUserUpdateForm {
    totp_2fa_enabled: Some(data.enabled),
    // if totp is enabled, leave unchanged. otherwise clear secret
    totp_2fa_secret: if data.enabled { None } else { Some(None) },
    ..Default::default()
  };

  LocalUser::update(
    &mut context.pool(),
    local_user_view.local_user.id,
    &local_user_form,
  )
  .await?;

  Ok(Json(UpdateTotpResponse {
    enabled: data.enabled,
  }))
}
