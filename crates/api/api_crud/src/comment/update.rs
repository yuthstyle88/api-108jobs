use activitypub_federation::config::Data;
use actix_web::web::{ Json};
use chrono::Utc;
use lemmy_api_utils::utils::check_community_deleted_removed;
use lemmy_api_utils::{
  build_response::{build_comment_response, send_local_notifs},
  context::FastJobContext,
  plugins::plugin_hook_after,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{get_url_blocklist, process_markdown_opt, slur_regex},
};
use lemmy_db_schema::{
  impls::actor_language::validate_post_language,
  source::comment::{Comment, CommentUpdateForm},
  traits::Crud,
};
use lemmy_db_views_comment::{
  api::{CommentResponse, EditComment},
  CommentView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{
  error::{FastJobErrorType, FastJobResult},
  utils::validation::is_valid_body_field,
};
use url;

pub async fn update_comment(
  data: Json<EditComment>,
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

  check_community_deleted_removed(&orig_comment.community)?;

  // Verify that only the creator can edit
  if local_user_view.person.id != orig_comment.creator.id {
    Err(FastJobErrorType::NoCommentEditAllowed)?
  }

  let language_id = validate_post_language(
    &mut context.pool(),
    data.language_id,
    orig_comment.community.id,
    local_user_view.local_user.id,
  )
  .await?;

  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  let content = process_markdown_opt(&data.content, &slur_regex, &url_blocklist, &context).await?;
  if let Some(content) = &content {
    is_valid_body_field(content, false)?;
  }

  // Validate required proposal fields (all comments now require these)
  validate_proposal_update_fields(&data)?;

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

fn validate_proposal_update_fields(data: &EditComment) -> FastJobResult<()> {
  // Validate budget (now required)
  if data.budget <= 0 {
    return Err(FastJobErrorType::InvalidField("budget must be greater than 0".to_string()))?;
  }

  // Validate working days (now required)
  if data.working_days <= 0 {
    return Err(FastJobErrorType::InvalidField("working_days must be greater than 0".to_string()))?;
  }

  // Validate brief URL (now required)
  if data.brief_url.trim().is_empty() {
    return Err(FastJobErrorType::InvalidField("brief_url cannot be empty".to_string()))?;
  }
  
  // Basic URL validation
  if let Err(_) = url::Url::parse(&data.brief_url) {
    return Err(FastJobErrorType::InvalidField("brief_url must be a valid URL".to_string()))?;
  }

  Ok(())
}
