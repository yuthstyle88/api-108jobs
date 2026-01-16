use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::{
  build_response::build_comment_response, context::FastJobContext, utils::check_private_instance,
};
use app_108jobs_db_views_comment::api::{CommentResponse, GetComment};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_utils::error::FastJobResult;

pub async fn get_comment(
  data: Query<GetComment>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<CommentResponse>> {
  let site_view = context.site_config().get().await?.site_view;
  let local_site = site_view.local_site;
  let local_instance_id = site_view.site.instance_id;

  check_private_instance(&local_user_view, &local_site)?;

  Ok(Json(
    build_comment_response(&context, data.id, local_user_view, local_instance_id).await?,
  ))
}
