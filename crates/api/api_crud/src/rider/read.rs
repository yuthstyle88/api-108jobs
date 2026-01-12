use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::{context::FastJobContext, utils::check_private_instance};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_rider::{
  api::{GetRider, GetRiderResponse},
  RiderView,
};
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};

pub async fn get_rider(
  data: Query<GetRider>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<GetRiderResponse>> {
  let site_view = context.site_config().get().await?.site_view;
  let local_site = site_view.local_site;

  // Private instance guard
  check_private_instance(&local_user_view, &local_site)?;

  // Resolve rider id
  let rider_id = if let Some(id) = data.id {
    id
  } else {
    Err(FastJobErrorType::NotFound)?
  };

  // Main view fetch
  let rider_view = RiderView::read(&mut context.pool(), rider_id).await?;

  Ok(Json(GetRiderResponse { rider_view }))
}
