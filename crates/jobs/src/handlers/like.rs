use actix_web::web::{Data, Json};
use app_108jobs_api_utils::{
  build_response::build_post_response, context::FastJobContext, utils::check_bot_account,
};
use app_108jobs_core::error::FastJobResult;
use app_108jobs_db::{
  source::{
    person::PersonActions,
    post::{PostActions, PostReadForm},
  },
  traits::{Likeable, Readable},
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::{
  api::{CreatePostLikeRequest, PostResponse},
  PostView,
};
use std::ops::Deref;

pub async fn like_post(
  data: Json<CreatePostLikeRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PostResponse>> {
  let local_instance_id = local_user_view.person.instance_id;
  let post_id = data.post_id;
  let my_person_id = local_user_view.person.id;

  check_bot_account(&local_user_view.person)?;

  // Check for a category ban
  let orig_post = PostView::read(&mut context.pool(), post_id, None, local_instance_id).await?;
  let previous_score = orig_post.post_actions.and_then(|p| p.like_score);

  // Remove any likes first
  PostActions::remove_like(&mut context.pool(), my_person_id, post_id).await?;
  if let Some(previous_score) = previous_score {
    PersonActions::remove_like(
      &mut context.pool(),
      my_person_id,
      orig_post.creator.id,
      previous_score,
    )
    .await
    // Ignore errors, since a previous_like of zero throws an error
    .ok();
  }

  // Mark Post Read
  let read_form = PostReadForm::new(post_id, my_person_id);
  PostActions::mark_as_read(&mut context.pool(), &read_form).await?;

  build_post_response(context.deref(), local_user_view, post_id).await
}
