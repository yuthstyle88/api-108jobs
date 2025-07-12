use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::FastJobContext, utils::is_admin};
use lemmy_db_schema::{
  source::oauth_provider::{OAuthProvider, OAuthProviderInsertForm},
  traits::Crud,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::CreateOAuthProvider;
use lemmy_utils::error::FastJobError;

pub async fn create_oauth_provider(
  data: Json<CreateOAuthProvider>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> Result<Json<OAuthProvider>, FastJobError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let cloned_data = data.clone();
  let oauth_provider_form = OAuthProviderInsertForm {
    display_name: cloned_data.display_name,
    auto_verify_email: data.auto_verify_email,
    account_linking_enabled: data.account_linking_enabled,
    enabled: data.enabled,
  };
  let oauth_provider = OAuthProvider::create(&mut context.pool(), &oauth_provider_form).await?;
  Ok(Json(oauth_provider))
}
