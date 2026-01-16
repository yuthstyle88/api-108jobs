use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::{source::post::PostActions, traits::Readable};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::api::MarkManyPostsAsRead;
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_utils::{error::FastJobResult, utils::validation::check_api_elements_count};

pub async fn mark_posts_as_read(
  data: Json<MarkManyPostsAsRead>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let post_ids = &data.post_ids;
  check_api_elements_count(post_ids.len())?;

  let person_id = local_user_view.person.id;

  let forms = PostActions::build_many_read_forms(post_ids, person_id);

  // Mark the posts as read
  PostActions::mark_many_as_read(&mut context.pool(), &forms).await?;

  Ok(Json(SuccessResponse::default()))
}
