use actix_web::web::Data;
use actix_web::web::Json;
use bcrypt::verify;
use app_108jobs_api_utils::{
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::purge_user_account,
};
use app_108jobs_db_schema::source::{
  category::CategoryActions,
  login_token::LoginToken,
  oauth_account::OAuthAccount,
  person::Person,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::{DeleteAccount, SuccessResponse};
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};

pub async fn delete_account(
  data: Json<DeleteAccount>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let local_instance_id = local_user_view.person.instance_id;

  // Verify the password
  let valid: bool = local_user_view
    .local_user
    .password_encrypted
    .as_ref()
    .and_then(|password_encrypted| verify(&data.password, password_encrypted).ok())
    .unwrap_or(false);
  if !valid {
    Err(FastJobErrorType::IncorrectLogin)?
  }

  if data.delete_content {
    purge_user_account(local_user_view.person.id, local_instance_id, &context).await?;
  } else {
    // These are already run in purge_user_account,
    // but should be done anyway even if delete_content is false
    OAuthAccount::delete_user_accounts(&mut context.pool(), local_user_view.local_user.id).await?;
    CategoryActions::leave_mod_team_for_all_communities(
      &mut context.pool(),
      local_user_view.person.id,
    )
    .await?;
    Person::delete_account(
      &mut context.pool(),
      local_user_view.person.id,
      local_instance_id,
    )
    .await?;
  }

  LoginToken::invalidate_all(&mut context.pool(), local_user_view.local_user.id).await?;

  ActivityChannel::submit_activity(
    SendActivityData::DeleteUser(local_user_view.person, data.delete_content),
    &context,
  )?;

  Ok(Json(SuccessResponse::default()))
}
