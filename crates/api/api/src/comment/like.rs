use actix_web::web::{Json};
use lemmy_api_utils::{
  build_response::build_comment_response,
  context::FastJobContext
  ,
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_db_schema::{
  source::{
    comment::CommentActions,
    person::PersonActions,
  },
  traits::Likeable,
};
use lemmy_db_views_comment::{
  api::{CommentResponse, CreateCommentLike},
  CommentView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;
use std::ops::Deref;
use activitypub_federation::config::Data;

pub async fn like_comment(
  data: Json<CreateCommentLike>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CommentResponse>> {
  let local_instance_id = local_user_view.person.instance_id;
  let comment_id = data.comment_id;
  let my_person_id = local_user_view.person.id;


  let orig_comment = CommentView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;
  let previous_score = orig_comment.comment_actions.and_then(|p| p.like_score);

  // Remove any likes first
  CommentActions::remove_like(&mut context.pool(), my_person_id, comment_id).await?;
  if let Some(previous_score) = previous_score {
    PersonActions::remove_like(
      &mut context.pool(),
      my_person_id,
      orig_comment.creator.id,
      previous_score,
    )
    .await
    // Ignore errors, since a previous_like of zero throws an error
    .ok();
  }

  ActivityChannel::submit_activity(
    SendActivityData::LikePostOrComment {
      actor: local_user_view.person.clone(),
      community: orig_comment.community,
      previous_score,
      new_score: data.score,
    },
    &context,
  )?;

  Ok(Json(
    build_comment_response(
      context.deref(),
      comment_id,
      Some(local_user_view),
      local_instance_id,
    )
    .await?,
  ))
}
