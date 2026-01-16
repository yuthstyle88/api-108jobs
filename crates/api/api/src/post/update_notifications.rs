use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::source::post::PostActions;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::api::UpdatePostNotifications;
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_utils::error::FastJobResult;

pub async fn update_post_notifications(
  data: Json<UpdatePostNotifications>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  PostActions::update_notification_state(
    data.post_id,
    local_user_view.person.id,
    data.new_state,
    &mut context.pool(),
  )
  .await?;
  Ok(Json(SuccessResponse::default()))
}
