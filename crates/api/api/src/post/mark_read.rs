use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::{
  source::post::{PostActions, PostReadForm},
  traits::Readable,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::{
  api::{MarkPostAsRead, PostResponse},
  PostView,
};
use app_108jobs_db_views_post::logistics::{self, LogisticsViewer};
use app_108jobs_utils::error::FastJobResult;

pub async fn mark_post_as_read(
  data: Json<MarkPostAsRead>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PostResponse>> {
  let person_id = local_user_view.person.id;
  let local_instance_id = local_user_view.person.instance_id;
  let post_id = data.post_id;

  // Mark the post as read / unread
  let form = PostReadForm::new(post_id, person_id);
  if data.read {
    PostActions::mark_as_read(&mut context.pool(), &form).await?;
  } else {
    PostActions::mark_as_unread(&mut context.pool(), &form).await?;
  }
  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;
  // Determine viewer and load logistics
  let is_admin = local_user_view.local_user.admin;
  let viewer = if is_admin {
    LogisticsViewer::Admin
  } else if local_user_view.person.id == post_view.creator.id {
    LogisticsViewer::Employer(post_view.creator.id)
  } else {
    LogisticsViewer::Public
  };

  let logistics = logistics::load_post_logistics(
    &mut context.pool(),
    post_id,
    post_view.post.post_kind,
    post_view.creator.id,
    viewer,
    is_admin,
  )
  .await?;

  Ok(Json(PostResponse { post_view, logistics }))
}
