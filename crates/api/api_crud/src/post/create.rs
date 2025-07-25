use super::convert_published_time;
use actix_web::web::{Data, Json};
use lemmy_api_utils::{
  build_response::{build_post_response, send_local_notifs},
  context::FastJobContext,
  plugins::{plugin_hook_after, plugin_hook_before},
  request::generate_post_link_metadata,
  send_activity::SendActivityData,
  tags::update_post_tags,
  utils::{
    check_community_user_action, check_self_promotion_allowed, get_url_blocklist, honeypot_check,
    process_markdown_opt, send_webmention, slur_regex,
  },
};
use lemmy_db_schema::{
  impls::actor_language::validate_post_language,
  source::post::{Post, PostActions, PostInsertForm, PostLikeForm, PostReadForm},
  traits::{Crud, Likeable, Readable},
  utils::diesel_url_create,
};
use lemmy_db_schema::newtypes::DbUrl;
use lemmy_db_views_community::CommunityView;
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::api::{CreatePost, CreatePostRequest, PostResponse};
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::FastJobResult,
  utils::{
    slurs::check_slurs,
    validation::{is_url_blocked, is_valid_alt_text_field, is_valid_body_field, is_valid_url},
  },
};

pub async fn create_post(
  data: Json<CreatePostRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PostResponse>> {
  let data: CreatePost = data.into_inner().try_into()?;
  honeypot_check(&data.honeypot)?;
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;

  let slur_regex = slur_regex(&context).await?;
  check_slurs(&data.name, &slur_regex)?;
  let url_blocklist = get_url_blocklist(&context).await?;

  let body = process_markdown_opt(&data.body, &slur_regex, &url_blocklist, &context).await?;
  let url = diesel_url_create(data.url.as_deref())?;
  let custom_thumbnail = diesel_url_create(data.custom_thumbnail.as_deref())?;
  check_self_promotion_allowed(data.self_promotion, Some(&local_site))?;

  if let Some(url) = &url {
    is_url_blocked(url, &url_blocklist)?;
    is_valid_url(url)?;
  }

  if let Some(custom_thumbnail) = &custom_thumbnail {
    is_valid_url(custom_thumbnail)?;
  }

  if let Some(alt_text) = &data.alt_text {
    is_valid_alt_text_field(alt_text)?;
  }

  if let Some(body) = &body {
    is_valid_body_field(body, true)?;
  }

  let community_view = CommunityView::read(
    &mut context.pool(),
    data.community_id,
    Some(&local_user_view.local_user),
  )
  .await?;
  let community = &community_view.community;
  check_community_user_action(&local_user_view, community, &mut context.pool()).await?;

  // Ensure that all posts in NSFW communities are marked as NSFW
  let self_promotion = if community.self_promotion {
    Some(true)
  } else {
    data.self_promotion
  };

  if community.posting_restricted_to_mods {
    let community_id = data.community_id;
    CommunityModeratorView::check_is_community_moderator(
      &mut context.pool(),
      community_id,
      local_user_view.local_user.person_id,
    )
    .await?;
  }

  let language_id = validate_post_language(
    &mut context.pool(),
    data.language_id,
    data.community_id,
    local_user_view.local_user.id,
  )
  .await?;

  let scheduled_publish_time_at =
    convert_published_time(data.scheduled_publish_time_at, &local_user_view, &context).await?;
  let mut post_form = PostInsertForm {
    url,
    body,
    alt_text: data.alt_text.clone(),
    self_promotion,
    language_id: Some(language_id),
    scheduled_publish_time_at,
    ap_id: Some("https://example.com/random".parse::<DbUrl>()?),
    ..PostInsertForm::new(
      data.name.trim().to_string(),
      local_user_view.person.id,
      data.community_id,
    )
  };

  post_form = plugin_hook_before("before_create_local_post", post_form).await?;

  let inserted_post = Post::create(&mut context.pool(), &post_form).await?;

  plugin_hook_after("after_create_local_post", &inserted_post)?;

  if let Some(tags) = &data.tags {
    update_post_tags(
      &context,
      &inserted_post,
      &community_view,
      tags,
      &local_user_view,
    )
    .await?;
  }

  let federate_post = if scheduled_publish_time_at.is_none() {
    send_webmention(inserted_post.clone(), community);
    |post| Some(SendActivityData::CreatePost(post))
  } else {
    |_| None
  };
  generate_post_link_metadata(
    inserted_post.clone(),
    custom_thumbnail.map(Into::into),
    federate_post,
    context.clone(),
  )
  .await?;

  // They like their own post by default
  let person_id = local_user_view.person.id;
  let post_id = inserted_post.id;
  let like_form = PostLikeForm::new(post_id, person_id, 1);

  PostActions::like(&mut context.pool(), &like_form).await?;

  let do_send_email = !local_site.disable_email_notifications;
  send_local_notifs(
    &inserted_post,
    None,
    &local_user_view.person,
    community,
    do_send_email,
    &context,
  )
  .await?;

  let read_form = PostReadForm::new(post_id, person_id);
  PostActions::mark_as_read(&mut context.pool(), &read_form).await?;

  build_post_response(&context, local_user_view, post_id).await
}
