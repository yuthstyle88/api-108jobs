use actix_web::web::{Data, Json, Path};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_rider::{
  api::{GetRider, GetRiderResponse},
  RiderView,
};
use app_108jobs_utils::error::FastJobResult;

pub async fn get_rider(
  data: Path<GetRider>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<GetRiderResponse>> {
  // Resolve rider
  let rider_view = if let Some(id) = data.id {
    // If id is provided, read by rider id
    RiderView::read(&mut context.pool(), id).await?
  } else {
    // Otherwise, read current rider profile by local user id
    RiderView::read_by_user_id(&mut context.pool(), local_user_view.local_user.id).await?
  };

  Ok(Json(GetRiderResponse { rider_view }))
}
