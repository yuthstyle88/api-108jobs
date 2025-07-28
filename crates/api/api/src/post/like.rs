use actix_web::web::{Data, Json};
use lemmy_api_utils::{
  build_response::build_post_response,
  context::FastJobContext
  ,
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_bot_account,
};
use lemmy_db_schema::{
  source::{
    person::PersonActions,
    post::{PostActions, PostReadForm},
  },
  traits::{Likeable, Readable},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::{
  api::{CreatePostLike, PostResponse},
  PostView,
};
use lemmy_utils::error::FastJobResult;
use std::ops::Deref;

pub async fn like_post(
  data: Json<CreatePostLike>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PostResponse>> {
  let local_instance_id = local_user_view.person.instance_id;
  let post_id = data.post_id;
  let my_person_id = local_user_view.person.id;

  check_bot_account(&local_user_view.person)?;

  // Check for a community ban
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

  ActivityChannel::submit_activity(
    SendActivityData::LikePostOrComment {
      actor: local_user_view.person.clone(),
      community: orig_post.community.clone(),
      previous_score,
      new_score: data.score,
    },
    &context,
  )?;

  build_post_response(context.deref(), local_user_view, post_id).await
}
