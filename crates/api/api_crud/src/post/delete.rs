use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::utils::check_category_deleted_removed;
use app_108jobs_api_utils::{
  build_response::build_post_response,
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use app_108jobs_db_schema::{
  source::{
    category::Category,
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::api::{DeletePost, PostResponse};
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};

pub async fn delete_post(
  data: Json<DeletePost>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PostResponse>> {
  let post_id = data.post_id;
  let orig_post = Post::read(&mut context.pool(), post_id).await?;

  // Dont delete it if its already been deleted.
  if orig_post.deleted == data.deleted {
    Err(FastJobErrorType::CouldntUpdatePost)?
  }

  // Check category if present
  if let Some(category_id) = orig_post.category_id {
    let category = Category::read(&mut context.pool(), category_id).await?;
    check_category_deleted_removed(&category)?;
  }

  // Verify that only the creator can delete
  if !Post::is_post_creator(local_user_view.person.id, orig_post.creator_id) {
    Err(FastJobErrorType::NoPostEditAllowed)?
  }

  // Update the post
  let post = Post::update(
    &mut context.pool(),
    data.post_id,
    &PostUpdateForm {
      deleted: Some(data.deleted),
      ..Default::default()
    },
  )
  .await?;

  ActivityChannel::submit_activity(
    SendActivityData::DeletePost(post, local_user_view.person.clone(), data.0),
    &context,
  )?;

  build_post_response(
    &context,
    local_user_view,
    data.post_id,
  )
  .await
}
