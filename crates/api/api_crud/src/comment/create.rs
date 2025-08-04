use actix_web::web::Data;
use lemmy_db_schema::newtypes::{PersonId, PostId};
use actix_web::web::{Json};
use lemmy_api_utils::utils::get_url_blocklist;
use lemmy_api_utils::{
  build_response::{build_comment_response, send_local_notifs},
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_post_deleted_or_removed, process_markdown, slur_regex, update_read_comments},
};
use lemmy_db_schema::{
  impls::actor_language::validate_post_language,
  source::{
    comment::{Comment, CommentActions, CommentInsertForm, CommentLikeForm},
  },
  traits::Likeable,
  utils::DbPool,
};
use lemmy_db_views_comment::api::{CommentResponse, CreateComment, CreateCommentRequest};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::PostView;
use lemmy_utils::{
  error::{FastJobErrorType, FastJobResult},
  utils::validation::is_valid_body_field,
};
use lemmy_db_schema::traits::Crud;
use lemmy_utils::error::FastJobError;

pub async fn create_comment(
  data: Json<CreateCommentRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CommentResponse>> {
  let request_data = data.into_inner();
  tracing::info!("Incoming comment request: {:#?}", request_data);

  let data: CreateComment = request_data.try_into()?;
  tracing::info!("Converted to CreateComment: {:#?}", data);
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

  let parent_opt = if let Some(parent_id) = data.parent_id {
    Comment::read(&mut context.pool(), parent_id).await.ok()
  } else {
    None
  };

  let language_id = validate_post_language(
    &mut context.pool(),
    data.language_id,
    community_id,
    local_user_view.local_user.id,
  )
  .await?;

  // Validate required proposal fields (all comments now require these)

  // Check if user is trying to comment on their own post
  if post.creator_id == local_user_view.person.id {
    return Err(FastJobError::from(FastJobErrorType::InvalidField("cannot comment on your own post".to_string())))?;
  }

  // Check if user has already commented on this post
  check_user_already_commented(&mut context.pool(), local_user_view.person.id, post_id).await?;

  let mut comment_form = CommentInsertForm::new(local_user_view.person.id, data.post_id, content.clone());
  comment_form.language_id = Some(language_id);

  // Add required proposal fields (all comments now require these)
  // Debug: Output the comment form structure
  tracing::info!("Comment form to be inserted: {:#?}", comment_form);
  let parent_path = parent_opt.clone().map(|t| t.path);
  // Create the comment
  let inserted_comment =
    Comment::create(&mut context.pool(), &comment_form, parent_path.as_ref()).await?;

  tracing::info!("Successfully inserted comment: {:#?}", inserted_comment);

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


async fn check_user_already_commented(
  pool: &mut DbPool<'_>,
  person_id: PersonId,
  _post_id: PostId
) -> FastJobResult<()> {
  use lemmy_db_schema_file::schema::comment::dsl::*;
  use lemmy_db_schema::utils::get_conn;
  use diesel::prelude::*;
  use diesel_async::RunQueryDsl;

  let conn = &mut get_conn(pool).await?;

  let existing_comment = comment
    .filter(creator_id.eq(person_id))
    .filter(lemmy_db_schema_file::schema::comment::post_id.eq(post_id))
    .filter(deleted.eq(false))
    .filter(removed.eq(false))
    .first::<Comment>(conn)
    .await
    .optional()
    .map_err(|_| FastJobError::from(FastJobErrorType::CouldntCreateComment))?;

  if existing_comment.is_some() {
    return Err(FastJobError::from(FastJobErrorType::InvalidField("You have already commented on this post".to_string())))?;
  }

  Ok(())
}
