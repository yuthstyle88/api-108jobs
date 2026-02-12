use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::{
  source::post::{PostActions, PostReadForm, PostSavedForm},
  traits::{Readable, Saveable},
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::{
  api::{PostResponse, SavePost},
  PostView,
};
use app_108jobs_db_views_post::logistics::{self, LogisticsViewer};
use app_108jobs_utils::error::FastJobResult;

pub async fn save_post(
  data: Json<SavePost>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PostResponse>> {
  let post_saved_form = PostSavedForm::new(data.post_id, local_user_view.person.id);

  if data.save {
    PostActions::save(&mut context.pool(), &post_saved_form).await?;
  } else {
    PostActions::unsave(&mut context.pool(), &post_saved_form).await?;
  }

  let post_id = data.post_id;
  let person_id = local_user_view.person.id;
  let local_instance_id = local_user_view.person.instance_id;
  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  let read_form = PostReadForm::new(post_id, person_id);
  PostActions::mark_as_read(&mut context.pool(), &read_form).await?;
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
