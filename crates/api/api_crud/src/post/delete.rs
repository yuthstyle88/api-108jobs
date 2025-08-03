use actix_web::web::{Data, Json};
use lemmy_api_utils::utils::check_community_deleted_removed;
use lemmy_api_utils::{
  build_response::build_post_response,
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_db_schema::{
  source::{
    community::Community,
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::api::{DeletePost, PostResponse};
use lemmy_utils::error::{FastJobErrorType, FastJobResult};

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

  let community = Community::read(&mut context.pool(), orig_post.community_id).await?;
  check_community_deleted_removed(&community)?;

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
