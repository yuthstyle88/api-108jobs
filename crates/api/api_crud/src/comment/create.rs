use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::utils::get_url_blocklist;
use app_108jobs_api_utils::{
  build_response::{build_comment_response, send_local_notifs},
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_post_deleted_or_removed, process_markdown, slur_regex, update_read_comments},
};
use app_108jobs_db_schema::{
  impls::actor_language::{validate_post_language, UNDETERMINED_ID},
  newtypes::{PersonId, PostId},
  source::comment::{Comment, CommentActions, CommentInsertForm, CommentLikeForm},
  traits::Crud,
  traits::Likeable,
  utils::DbPool,
};
use app_108jobs_db_schema_file::schema::comment::{creator_id, deleted, post_id, removed};
use app_108jobs_db_schema_file::schema::comment::dsl::comment;
use app_108jobs_db_schema_file::enums::PostKind;
use app_108jobs_db_views_comment::api::{CommentResponse, CreateComment, CreateCommentRequest};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::PostView;
use app_108jobs_utils::error::FastJobError;
use app_108jobs_utils::{
  error::{FastJobErrorType, FastJobResult},
  utils::validation::is_valid_body_field,
};

pub async fn create_comment(
  data: Json<CreateCommentRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CommentResponse>> {
  let request_data = data.into_inner();

  let data: CreateComment = request_data.try_into()?;
  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  let content = process_markdown(&data.content, &slur_regex, &url_blocklist, &context).await?;
  is_valid_body_field(&content, false)?;

  let local_instance_id = local_user_view.person.instance_id;

  // Read the full post view in order to get the comments count.
  let post_view = PostView::read(
    &mut context.pool(),
    data.post_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  let post = post_view.post;

  // Handle delivery/ridetaxi posts which have no category
  let language_id = if let Some(category) = post_view.category {
    // Normal post with a category - validate language
    validate_post_language(
      &mut context.pool(),
      data.language_id,
      category.id,
      local_user_view.local_user.id,
    )
    .await?
  } else if matches!(post.post_kind, PostKind::Delivery | PostKind::RideTaxi) {
    // Delivery or RideTaxi post without a category - use provided language or UNDETERMINED_ID
    data.language_id.unwrap_or(UNDETERMINED_ID)
  } else {
    // Non-delivery/ridetaxi post without a category should not happen
    return Err(FastJobErrorType::NotFound)?;
  };

  check_post_deleted_or_removed(&post)?;

  // Check if post is locked, no new comments
  if post.locked {
    Err(FastJobErrorType::Locked)?
  }

  // Check if user is trying to comment on their own post
  if post.creator_id == local_user_view.person.id {
    return Err(FastJobErrorType::CannotCommentOnOwnPost)?;
  }

  // Check if user has already commented on this post
  check_user_already_commented(&mut context.pool(), local_user_view.person.id, data.post_id.clone()).await?;

  let comment_form = CommentInsertForm {
    language_id: Some(language_id),
    ..CommentInsertForm::new(local_user_view.person.id, data.post_id, content.clone())
  };

  // Create the comment
  let inserted_comment = Comment::create(&mut context.pool(), &comment_form).await?;

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
    data.post_id.clone(),
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
  current_post_id: PostId,
) -> FastJobResult<()> {
  use diesel::prelude::*;
  use diesel_async::RunQueryDsl;
  use app_108jobs_db_schema::utils::get_conn;

  let conn = &mut get_conn(pool).await?;

  let existing_comment = comment
    .filter(creator_id.eq(person_id))
    .filter(post_id.eq(current_post_id))
    .filter(deleted.eq(false))
    .filter(removed.eq(false))
    .first::<Comment>(conn)
    .await
    .optional()
    .map_err(|_| FastJobError::from(FastJobErrorType::DatabaseError))?;

  if existing_comment.is_some() {
    return Err(FastJobErrorType::AlreadyCommented)?;
  }

  Ok(())
}
