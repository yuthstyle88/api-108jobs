use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::{
  source::post::{PostActions, PostHideForm},
  traits::Hideable,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::{
  api::{HidePost, PostResponse},
  PostView,
};
use app_108jobs_db_views_post::logistics::{self, LogisticsViewer};
use app_108jobs_utils::error::FastJobResult;

pub async fn hide_post(
  data: Json<HidePost>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PostResponse>> {
  let person_id = local_user_view.person.id;
  let local_instance_id = local_user_view.person.instance_id;
  let post_id = data.post_id;

  let hide_form = PostHideForm::new(post_id, person_id);

  // Mark the post as hidden / unhidden
  if data.hide {
    PostActions::hide(&mut context.pool(), &hide_form).await?;
  } else {
    PostActions::unhide(&mut context.pool(), &hide_form).await?;
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
