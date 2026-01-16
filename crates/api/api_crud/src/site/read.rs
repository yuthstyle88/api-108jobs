use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::{GetSiteResponse, SuccessResponse};
use app_108jobs_db_views_site::SiteView;
use app_108jobs_utils::error::FastJobResult;

pub async fn get_site(
  local_user_view: Option<LocalUserView>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<GetSiteResponse>> {
  let mut snap = context
    .site_config()
    .get()
    .await
    .map_err(|e| anyhow::anyhow!("Failed to load site config: {e}"))?;
  
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  snap.site_view = site_view;

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