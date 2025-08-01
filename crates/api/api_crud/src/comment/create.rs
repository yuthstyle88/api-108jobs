use actix_web::web::{Data, Json};
use lemmy_api_utils::utils::get_url_blocklist;
use lemmy_api_utils::{
  build_response::{build_comment_response, send_local_notifs},
  context::FastJobContext,
  plugins::plugin_hook_after,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_post_deleted_or_removed, process_markdown, slur_regex, update_read_comments},
};
use lemmy_db_schema::{
  impls::actor_language::validate_post_language,
  source::comment::{Comment, CommentActions, CommentInsertForm, CommentLikeForm},
  traits::{Likeable},
};
use lemmy_db_views_comment::api::{CommentResponse, CreateComment, CreateCommentRequest};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::PostView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::{FastJobErrorType, FastJobResult},
  utils::validation::is_valid_body_field,
};

pub async fn create_comment(
  data: Json<CreateCommentRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CommentResponse>> {
  let data: CreateComment = data.into_inner().try_into()?;
  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  let content = process_markdown(&data.content, &slur_regex, &url_blocklist, &context).await?;
  is_valid_body_field(&content, false)?;

  // Check for a community ban
  let post_id = data.post_id;

  let local_instance_id = local_user_view.person.instance_id;

  // Read the full post view in order to get the comments count.
  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  let post = post_view.post;
  let community_id = post_view.community.id;

  check_post_deleted_or_removed(&post)?;

  // Check if post is locked, no new comments
  if post.locked {
    Err(FastJobErrorType::Locked)?
  }

  let language_id = validate_post_language(
    &mut context.pool(),
    data.language_id,
    community_id,
    local_user_view.local_user.id,
  )
  .await?;

  let comment_form = CommentInsertForm {
    language_id: Some(language_id),
    ..CommentInsertForm::new(local_user_view.person.id, data.post_id, content.clone())
  };

  // Create the comment
  let inserted_comment = Comment::create(&mut context.pool(), &comment_form).await?;
  plugin_hook_after("after_create_local_comment", &inserted_comment)?;
  send_local_notifs(
    &post,
    Some(&inserted_comment),
    &local_user_view.person,
    &context,
  )
  .await?;

  // You like your own comment by default
  let like_form = CommentLikeForm::new(local_user_view.person.id, inserted_comment.id, 1);

  CommentActions::like(&mut context.pool(), &like_form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::CreateComment(inserted_comment.clone()),
    &context,
  )?;

  // Update the read comments, so your own new comment doesn't appear as a +1 unread
  update_read_comments(
    local_user_view.person.id,
    post_id,
    post.comments + 1,
    &mut context.pool(),
  )
  .await?;

  // If we're responding to a comment where we're the recipient,
  // (ie we're the grandparent, or the recipient of the parent comment_reply),
  // then mark the parent as read.
  // Then we don't have to do it manually after we respond to a comment.

  Ok(Json(
    build_comment_response(
      &context,
      inserted_comment.id,
      Some(local_user_view),
      local_instance_id,
    )
    .await?,
  ))
}
