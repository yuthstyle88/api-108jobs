use super::convert_published_time;
use actix_web::web::Data;
use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_utils::{
  build_response::{build_post_response, send_local_notifs},
  context::FastJobContext,
  request::generate_post_link_metadata,
  send_activity::SendActivityData,
  tags::update_post_tags,
  utils::{
    check_self_promotion_allowed,
    get_url_blocklist,
    process_markdown_opt,
    slur_regex,
  },
};
use lemmy_db_schema::{
  impls::actor_language::validate_post_language,
  source::post::{Post, PostUpdateForm},
  traits::Crud,
  utils::{diesel_string_update, diesel_url_update},
};
use lemmy_db_views_community::CommunityView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::api::EditPostRequest;
use lemmy_db_views_post::{
  api::{EditPost, PostResponse},
  PostView,
};
use lemmy_utils::{
  error::{FastJobErrorType, FastJobResult},
  utils::{
    slurs::check_slurs,
    validation::{
      is_url_blocked,
      is_valid_alt_text_field,
      is_valid_body_field,
      is_valid_post_title,
      is_valid_url,
    },
  },
};
use std::ops::Deref;

pub async fn update_post(
  data: Json<EditPostRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PostResponse>> {
  let data: EditPost = data.into_inner().try_into()?;
  let local_site = context.site_config().get().await?.site_view.local_site;
  let local_instance_id = local_user_view.person.instance_id;
  let url = diesel_url_update(data.url.as_deref())?;

  let custom_thumbnail = diesel_url_update(data.custom_thumbnail.as_deref())?;

  let url_blocklist = get_url_blocklist(&context).await?;

  let slur_regex = slur_regex(&context).await?;

  let body = diesel_string_update(
    process_markdown_opt(&data.body, &slur_regex, &url_blocklist, &context)
      .await?
      .as_deref(),
  );

  check_self_promotion_allowed(data.self_promotion, Some(&local_site))?;

  let alt_text = diesel_string_update(data.alt_text.as_deref());

  if let Some(name) = &data.name {
    is_valid_post_title(name)?;
    check_slurs(name, &slur_regex)?;
  }

  if let Some(Some(body)) = &body {
    is_valid_body_field(body, true)?;
  }

  if let Some(Some(alt_text)) = &alt_text {
    is_valid_alt_text_field(alt_text)?;
  }

  if let Some(Some(url)) = &url {
    is_url_blocked(url, &url_blocklist)?;
    is_valid_url(url)?;
  }

  if let Some(Some(custom_thumbnail)) = &custom_thumbnail {
    is_valid_url(custom_thumbnail)?;
  }

  let post_id = data.post_id;
  let orig_post =
    PostView::read(&mut context.pool(), post_id, None, local_instance_id).await?;

  if let Some(tags) = &data.tags {
    // post view does not include communityview.post_tags
    let community_view =
      CommunityView::read(&mut context.pool(), orig_post.community.id, None).await?;
    update_post_tags(
      &context,
      &orig_post.post,
      &community_view,
      tags,
      &local_user_view,
    )
    .await?;
  }

  // Verify that only the creator can edit
  if !Post::is_post_creator(local_user_view.person.id, orig_post.post.creator_id) {
    Err(FastJobErrorType::NoPostEditAllowed)?
  }

  let language_id = validate_post_language(
    &mut context.pool(),
    data.language_id,
    orig_post.post.community_id,
    local_user_view.local_user.id,
  )
  .await?;

  // handle changes to scheduled_publish_time
  let scheduled_publish_time_at = match (
    orig_post.post.scheduled_publish_time_at,
    data.scheduled_publish_time_at,
  ) {
    // schedule time can be changed if post is still scheduled (and not published yet)
    (Some(_), Some(_)) => Some(
      convert_published_time(data.scheduled_publish_time_at, &local_user_view, &context).await?,
    ),
    // post was scheduled, gets changed to publish immediately
    (Some(_), None) => Some(None),
    // unchanged
    (_, _) => None,
  };

  let post_form = PostUpdateForm {
    name: data.name.clone(),
    community_id: data.community_id.clone(),
    url,
    body,
    alt_text,
    self_promotion: data.self_promotion,
    language_id: Some(language_id),
    updated_at: Some(Some(Utc::now())),
    scheduled_publish_time_at,
    intended_use: data.intended_use,
    job_type: data.job_type,
    budget: data.budget,
    deadline: Some(data.deadline),
    is_english_required: data.is_english_required,
    ..Default::default()
  };

  let post_id = data.post_id;
  let updated_post = Post::update(&mut context.pool(), post_id, &post_form).await?;

  send_local_notifs(
    &updated_post,
    None,
    &local_user_view.person,
    &context,
  )
  .await?;

  // send out federation/webmention if necessary
  match (
    orig_post.post.scheduled_publish_time_at,
    data.scheduled_publish_time_at,
  ) {
    // schedule was removed, send create activity and webmention
    (Some(_), None) => {
      generate_post_link_metadata(
        updated_post.clone(),
        custom_thumbnail.flatten().map(Into::into),
        |post| Some(SendActivityData::CreatePost(post)),
        context.clone(),
      )
      .await?;
    }
    // post was already public, send update
    (None, _) => {
      generate_post_link_metadata(
        updated_post.clone(),
        custom_thumbnail.flatten().map(Into::into),
        |post| Some(SendActivityData::UpdatePost(post)),
        context.clone(),
      )
      .await?
    }
    // schedule was changed, do nothing
    (Some(_), Some(_)) => {}
  };

  build_post_response(
    context.deref(),
    local_user_view,
    post_id,
  )
  .await
}
