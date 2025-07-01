use actix_web::web::{Data, Json};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::post::PostActions;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::api::UpdatePostNotifications;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::FastJobResult;

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
