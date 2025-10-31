use actix_web::web::Data;
use actix_web::web::Json;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::{GetSiteResponse, SuccessResponse};
use lemmy_utils::error::FastJobResult;

pub async fn get_site(
  local_user_view: Option<LocalUserView>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<GetSiteResponse>> {
  let snap = context
    .site_config()
    .get()
    .await
    .map_err(|e| anyhow::anyhow!("Failed to load site config: {e}"))?;

  let mut site_response: GetSiteResponse = snap.into();

  // Filter admin-only oauth providers for non-admin users
  if !local_user_view
    .map(|l| l.local_user.admin)
    .unwrap_or_default()
  {
    site_response.admin_oauth_providers.clear();
  }

  Ok(Json(site_response))
}

pub async fn health() -> FastJobResult<Json<SuccessResponse>> {
  Ok(Json(SuccessResponse{success: true}))
}