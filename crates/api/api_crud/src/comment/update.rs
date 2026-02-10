use actix_web::web::Data;
use actix_web::web::Json;
use chrono::Utc;
use app_108jobs_api_utils::utils::check_category_deleted_removed;
use app_108jobs_api_utils::{
  build_response::{build_comment_response, send_local_notifs},
  context::FastJobContext,
  plugins::plugin_hook_after,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{get_url_blocklist, process_markdown_opt, slur_regex},
};
use app_108jobs_db_schema::{
  impls::actor_language::validate_post_language,
  source::comment::{Comment, CommentUpdateForm},
  traits::Crud,
};
use app_108jobs_db_views_comment::{
  api::{CommentResponse, EditCommentRequest},
  CommentView,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_utils::{
  error::{FastJobErrorType, FastJobResult},
  utils::validation::is_valid_body_field,
};

pub async fn update_comment(
  data: Json<EditCommentRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CommentResponse>> {
  let comment_id = data.comment_id;
  let local_instance_id = local_user_view.person.instance_id;
  let orig_comment = CommentView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  check_category_deleted_removed(
    orig_comment
      .category
      .as_ref()
      .ok_or(FastJobErrorType::NotFound)?,
  )?;

  // Verify that only the creator can edit
  if local_user_view.person.id != orig_comment.creator.id {
    Err(FastJobErrorType::NoCommentEditAllowed)?
  }

  let language_id = validate_post_language(
    &mut context.pool(),
    data.language_id,
    orig_comment
      .category
      .as_ref()
      .ok_or(FastJobErrorType::NotFound)?
      .id,
    local_user_view.local_user.id,
  )
  .await?;

  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  let content = process_markdown_opt(&data.content, &slur_regex, &url_blocklist, &context).await?;
  if let Some(content) = &content {
    is_valid_body_field(content, false)?;
  }


  let comment_id = data.comment_id;
  let form = CommentUpdateForm {
    content,
    language_id: Some(language_id),
    updated_at: Some(Some(Utc::now())),
    ..Default::default()
  };
  let updated_comment = Comment::update(&mut context.pool(), comment_id, &form).await?;

  plugin_hook_after("after_update_local_comment", &updated_comment)?;

  // Do the mentions / recipients
  send_local_notifs(
    &orig_comment.post,
    Some(&updated_comment),
    &local_user_view.person,
    &context,
  )
  .await?;

  ActivityChannel::submit_activity(
    SendActivityData::UpdateComment(updated_comment.clone()),
    &context,
  )?;

  Ok(Json(
    build_comment_response(
      &context,
      updated_comment.id,
      Some(local_user_view),
      local_instance_id,
    )
    .await?,
  ))
}

