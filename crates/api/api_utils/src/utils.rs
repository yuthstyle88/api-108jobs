use crate::{
  claims::Claims,
  context::FastJobContext,
  request::{delete_image_alias, fetch_pictrs_proxied_image_details, purge_image_from_pictrs_url},
};
use actix_web::{http::header::Header, HttpRequest};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use chrono::{DateTime, Days, Local, TimeZone, Utc};
use enum_map::{enum_map, EnumMap};
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, DbUrl, InstanceId, PersonId, PostId},
  source::{
    comment::{Comment, CommentActions},
    community::{Community, CommunityActions, CommunityUpdateForm},
    images::{ImageDetails, RemoteImage},
    local_site::LocalSite,
    local_site_rate_limit::LocalSiteRateLimit,
    local_site_url_blocklist::LocalSiteUrlBlocklist,
    mod_log::moderator::{
      ModRemoveComment,
      ModRemoveCommentForm,
      ModRemovePost,
      ModRemovePostForm,
    },
    oauth_account::OAuthAccount,
    person::{Person, PersonUpdateForm},
    post::{Post, PostActions, PostReadCommentsForm},
    registration_application::RegistrationApplication,
    site::Site,
  },
  traits::{Crud, Likeable, ReadComments},
  utils::DbPool,
};
use lemmy_db_schema_file::enums::RegistrationMode;
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_image::LocalImageView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::PersonView;
use lemmy_db_views_site::{
  SiteView,
};
use lemmy_utils::{
  error::{FastJobError, FastJobErrorExt, FastJobErrorExt2, FastJobErrorType, FastJobResult},
  rate_limit::{ActionType, BucketConfig},
  settings::{structs::PictrsImageMode, SETTINGS},
  spawn_try_task,
  utils::{
    markdown::{image_links::markdown_rewrite_image_links, markdown_check_for_blocked_urls},
    slurs::remove_slurs,
    validation::{build_and_check_regex, clean_urls_in_text},
  },
  CacheLock,
  CACHE_DURATION_FEDERATION,
};
use moka::future::Cache;
use regex::{escape, Regex, RegexSet};
use std::sync::LazyLock;

use tracing::Instrument;
use url::{ParseError, Url};
use urlencoding::encode;
use webmention::{Webmention, WebmentionError};
use rand::Rng;

pub const AUTH_COOKIE_NAME: &str = "jwt";

/// Checks if a person is an admin, or moderator of any community.
pub(crate) async fn check_is_mod_of_any_or_admin(
  pool: &mut DbPool<'_>,
  person_id: PersonId,
  local_instance_id: InstanceId,
) -> FastJobResult<()> {
  let is_mod_of_any = CommunityModeratorView::is_community_moderator_of_any(pool, person_id).await;
  if is_mod_of_any.is_ok()
    || PersonView::read(pool, person_id, None, local_instance_id, false)
      .await
      .is_ok_and(|t| t.is_admin)
  {
    Ok(())
  } else {
    Err(FastJobErrorType::NotAModOrAdmin)?
  }
}

/// Check that a person is either a mod of any community, or an admin
///
/// Should only be used for read operations
pub async fn check_community_mod_of_any_or_admin_action(
  local_user_view: &LocalUserView,
  pool: &mut DbPool<'_>,
) -> FastJobResult<()> {
  let person = &local_user_view.person;

  check_local_user_valid(local_user_view)?;
  check_is_mod_of_any_or_admin(pool, person.id, person.instance_id).await
}

pub fn is_admin(local_user_view: &LocalUserView) -> FastJobResult<()> {
  check_local_user_valid(local_user_view)?;
  if !local_user_view.local_user.admin {
    Err(FastJobErrorType::NotAnAdmin)?
  } else {
    Ok(())
  }
}

/// Updates the read comment count for a post. Usually done when reading or creating a new comment.
pub async fn update_read_comments(
  person_id: PersonId,
  post_id: PostId,
  read_comments: i64,
  pool: &mut DbPool<'_>,
) -> FastJobResult<()> {
  let person_post_agg_form = PostReadCommentsForm::new(post_id, person_id, read_comments);
  PostActions::update_read_comments(pool, &person_post_agg_form).await?;

  Ok(())
}

pub fn check_local_user_valid(local_user_view: &LocalUserView) -> FastJobResult<()> {
  // Check for a site ban
  if local_user_view.banned {
    Err(FastJobErrorType::SiteBan)?
  }
  check_local_user_deleted(local_user_view)
}

/// Check for account deletion
pub fn check_local_user_deleted(local_user_view: &LocalUserView) -> FastJobResult<()> {
  if local_user_view.person.deleted {
    Err(FastJobErrorType::Deleted)?
  } else {
    Ok(())
  }
}

pub fn check_person_valid(person_view: &PersonView) -> FastJobResult<()> {
  // Check for a site ban
  if person_view.creator_banned {
    Err(FastJobErrorType::SiteBan)?
  }
  // check for account deletion
  else if person_view.person.deleted {
    Err(FastJobErrorType::Deleted)?
  } else {
    Ok(())
  }
}

/// Check if the user's multilang is verified if multilang verification is turned on
/// However, skip checking verification if the user is an admin
pub fn check_email_verified(
  local_user_view: &LocalUserView,
  site_view: &SiteView,
) -> FastJobResult<()> {
  if !local_user_view.local_user.admin
    && site_view.local_site.require_email_verification
    && !local_user_view.local_user.email_verified
  {
    Err(FastJobErrorType::EmailNotVerified)?
  }
  Ok(())
}

pub async fn check_registration_application(
  local_user_view: &LocalUserView,
  local_site: &LocalSite,
  pool: &mut DbPool<'_>,
) -> FastJobResult<()> {
  if (local_site.registration_mode == RegistrationMode::RequireApplication
    || local_site.registration_mode == RegistrationMode::Closed)
    && !local_user_view.local_user.accepted_application
    && !local_user_view.local_user.admin
  {
    // Fetch the registration application. If no admin id is present its still pending. Otherwise it
    // was processed (either accepted or denied).
    let local_user_id = local_user_view.local_user.id;
    let registration = RegistrationApplication::find_by_local_user_id(pool, local_user_id).await?;
    if registration.admin_id.is_some() {
      Err(FastJobErrorType::RegistrationDenied {
        reason: registration.deny_reason,
      })?
    } else {
      Err(FastJobErrorType::RegistrationApplicationIsPending)?
    }
  }
  Ok(())
}

pub fn check_community_deleted_removed(community: &Community) -> FastJobResult<()> {
  if community.deleted || community.removed {
    Err(FastJobErrorType::AlreadyDeleted)?
  }
  Ok(())
}

/// Don't allow creating reports for removed / deleted posts
pub fn check_post_deleted_or_removed(post: &Post) -> FastJobResult<()> {
  if post.deleted || post.removed {
    Err(FastJobErrorType::Deleted)?
  } else {
    Ok(())
  }
}

pub fn check_comment_deleted_or_removed(comment: &Comment) -> FastJobResult<()> {
  if comment.deleted || comment.removed {
    Err(FastJobErrorType::Deleted)?
  } else {
    Ok(())
  }
}

/// Dont allow bots to do certain actions, like voting
pub fn check_bot_account(person: &Person) -> FastJobResult<()> {
  if person.bot_account {
    Err(FastJobErrorType::InvalidBotAction)?
  } else {
    Ok(())
  }
}

pub fn check_private_instance(
  local_user_view: &Option<LocalUserView>,
  local_site: &LocalSite,
) -> FastJobResult<()> {
  if local_user_view.is_none() && local_site.private_instance {
    Err(FastJobErrorType::InstanceIsPrivate)?
  } else {
    Ok(())
  }
}

/// Checks for a honeypot. If this field is filled, fail the rest of the function
pub fn honeypot_check(honeypot: &Option<String>) -> FastJobResult<()> {
  if honeypot.is_some() && honeypot != &Some(String::new()) {
    Err(FastJobErrorType::HoneypotFailed)?
  } else {
    Ok(())
  }
}

pub fn local_site_rate_limit_to_rate_limit_config(
  l: &LocalSiteRateLimit,
) -> EnumMap<ActionType, BucketConfig> {
  enum_map! {
    ActionType::Message => (l.message_max_requests, l.message_interval_seconds),
    ActionType::Post => (l.post_max_requests, l.post_interval_seconds),
    ActionType::Register => (l.register_max_requests, l.register_interval_seconds),
    ActionType::Image => (l.image_max_requests, l.image_interval_seconds),
    ActionType::Comment => (l.comment_max_requests, l.comment_interval_seconds),
    ActionType::Search => (l.search_max_requests, l.search_interval_seconds),
    ActionType::ImportUserSettings => (l.import_user_settings_max_requests, l.import_user_settings_interval_seconds),
  }
  .map(|_key, (max_requests, interval)| BucketConfig {
    max_requests: u32::try_from(max_requests).unwrap_or(0),
    interval: u32::try_from(interval).unwrap_or(0),
  })
}

pub async fn slur_regex(context: &FastJobContext) -> FastJobResult<Regex> {
  static CACHE: CacheLock<Regex> = LazyLock::new(|| {
    Cache::builder()
      .max_capacity(1)
      .time_to_live(CACHE_DURATION_FEDERATION)
      .build()
  });
  Ok(
    CACHE
      .try_get_with((), async {
        let local_site = SiteView::read_local(&mut context.pool())
          .await
          .ok()
          .map(|s| s.local_site);
        build_and_check_regex(local_site.and_then(|s| s.slur_filter_regex).as_deref())
      })
      .await
      .map_err(|e| anyhow::anyhow!("Failed to construct regex: {e}"))?,
  )
}

pub async fn get_url_blocklist(context: &FastJobContext) -> FastJobResult<RegexSet> {
  static URL_BLOCKLIST: CacheLock<RegexSet> = LazyLock::new(|| {
    Cache::builder()
      .max_capacity(1)
      .time_to_live(CACHE_DURATION_FEDERATION)
      .build()
  });

  Ok(
    URL_BLOCKLIST
      .try_get_with::<_, FastJobError>((), async {
        let urls = LocalSiteUrlBlocklist::get_all(&mut context.pool()).await?;

        // The urls are already validated on saving, so just escape them.
        // If this regex creation changes it must be synced with
        // lemmy_utils::utils::markdown::create_url_blocklist_test_regex_set.
        let regexes = urls.iter().map(|url| format!(r"\b{}\b", escape(&url.url)));

        let set = RegexSet::new(regexes)?;
        Ok(set)
      })
      .await
      .map_err(|e| anyhow::anyhow!("Failed to build URL blocklist due to `{}`", e))?,
  )
}

pub fn check_self_promotion_allowed(self_promotion: Option<bool>, local_site: Option<&LocalSite>) -> FastJobResult<()> {
  let is_self_promotion = self_promotion.unwrap_or_default();
  let self_promotion_disallowed = local_site.is_some_and(|s| s.disallow_self_promotion_content);

  if self_promotion_disallowed && is_self_promotion {
    Err(FastJobErrorType::NsfwNotAllowed)?
  }

  Ok(())
}

/// Read the site for an ap_id.
///
/// Used for GetCommunityResponse and GetPersonDetails
pub async fn read_site_for_actor(
  ap_id: DbUrl,
  context: &FastJobContext,
) -> FastJobResult<Option<Site>> {
  let site_id = Site::instance_ap_id_from_url(ap_id.clone().into());
  let site = Site::read_from_apub_id(&mut context.pool(), &site_id.into()).await?;
  Ok(site)
}

pub async fn purge_post_images(
  url: Option<DbUrl>,
  thumbnail_url: Option<DbUrl>,
  context: &FastJobContext,
) {
  if let Some(url) = url {
    purge_image_from_pictrs_url(&url, context).await.ok();
  }
  if let Some(thumbnail_url) = thumbnail_url {
    purge_image_from_pictrs_url(&thumbnail_url, context)
      .await
      .ok();
  }
}

/// Delete local images attributed to a person
async fn delete_local_user_images(person_id: PersonId, context: &FastJobContext) -> FastJobResult<()> {
  let pictrs_uploads = LocalImageView::get_all_by_person_id(&mut context.pool(), person_id).await?;

  // Delete their images
  for upload in pictrs_uploads {
    delete_image_alias(&upload.local_image.pictrs_alias, context)
      .await
      .ok();
  }
  Ok(())
}

/// Removes or restores user data.
pub async fn remove_or_restore_user_data(
  mod_person_id: PersonId,
  banned_person_id: PersonId,
  removed: bool,
  reason: &Option<String>,
  context: &FastJobContext,
) -> FastJobResult<()> {
  let pool = &mut context.pool();

  // These actions are only possible when removing, not restoring
  if removed {
    delete_local_user_images(banned_person_id, context).await?;

    // Update the fields to None
    Person::update(
      pool,
      banned_person_id,
      &PersonUpdateForm {
        avatar: Some(None),
        banner: Some(None),
        bio: Some(None),
        ..Default::default()
      },
    )
    .await?;

    // Communities
    // Remove all communities where they're the top mod
    // for now, remove the communities manually
    let first_mod_communities = CommunityModeratorView::get_community_first_mods(pool).await?;

    // Filter to only this banned users top communities
    let banned_user_first_communities: Vec<CommunityModeratorView> = first_mod_communities
      .into_iter()
      .filter(|fmc| fmc.moderator.id == banned_person_id)
      .collect();

    for first_mod_community in banned_user_first_communities {
      let community_id = first_mod_community.community.id;
      Community::update(
        pool,
        community_id,
        &CommunityUpdateForm {
          removed: Some(removed),
          ..Default::default()
        },
      )
      .await?;

      // Update the fields to None
      Community::update(
        pool,
        community_id,
        &CommunityUpdateForm {
          icon: Some(None),
          banner: Some(None),
          ..Default::default()
        },
      )
      .await?;
    }

    // Remove post and comment votes
    PostActions::remove_all_likes(pool, banned_person_id).await?;
    CommentActions::remove_all_likes(pool, banned_person_id).await?;
  }

  // Posts
  let removed_or_restored_posts =
    Post::update_removed_for_creator(pool, banned_person_id, removed).await?;
  create_modlog_entries_for_removed_or_restored_posts(
    pool,
    mod_person_id,
    removed_or_restored_posts.iter().map(|r| r.id).collect(),
    removed,
    reason,
  )
  .await?;

  // Comments
  let removed_or_restored_comments =
    Comment::update_removed_for_creator(pool, banned_person_id, removed).await?;
  create_modlog_entries_for_removed_or_restored_comments(
    pool,
    mod_person_id,
    removed_or_restored_comments.iter().map(|r| r.id).collect(),
    removed,
    reason,
  )
  .await?;

  Ok(())
}

async fn create_modlog_entries_for_removed_or_restored_posts(
  pool: &mut DbPool<'_>,
  mod_person_id: PersonId,
  post_ids: Vec<PostId>,
  removed: bool,
  reason: &Option<String>,
) -> FastJobResult<()> {
  // Build the forms
  let forms = post_ids
    .iter()
    .map(|&post_id| ModRemovePostForm {
      mod_person_id,
      post_id,
      removed: Some(removed),
      reason: reason.clone(),
    })
    .collect();

  ModRemovePost::create_multiple(pool, &forms).await?;

  Ok(())
}

async fn create_modlog_entries_for_removed_or_restored_comments(
  pool: &mut DbPool<'_>,
  mod_person_id: PersonId,
  comment_ids: Vec<CommentId>,
  removed: bool,
  reason: &Option<String>,
) -> FastJobResult<()> {
  // Build the forms
  let forms = comment_ids
    .iter()
    .map(|&comment_id| ModRemoveCommentForm {
      mod_person_id,
      comment_id,
      removed: Some(removed),
      reason: reason.clone(),
    })
    .collect();

  ModRemoveComment::create_multiple(pool, &forms).await?;

  Ok(())
}

pub async fn remove_or_restore_user_data_in_community(
  community_id: CommunityId,
  mod_person_id: PersonId,
  banned_person_id: PersonId,
  remove: bool,
  reason: &Option<String>,
  pool: &mut DbPool<'_>,
) -> FastJobResult<()> {
  // These actions are only possible when removing, not restoring
  if remove {
    // Remove post and comment votes
    PostActions::remove_likes_in_community(pool, banned_person_id, community_id).await?;
    CommentActions::remove_likes_in_community(pool, banned_person_id, community_id).await?;
  }

  // Posts
  let posts =
    Post::update_removed_for_creator_and_community(pool, banned_person_id, community_id, remove)
      .await?;

  create_modlog_entries_for_removed_or_restored_posts(
    pool,
    mod_person_id,
    posts.iter().map(|r| r.id).collect(),
    remove,
    reason,
  )
  .await?;

  // Comments
  let removed_comment_ids =
    Comment::update_removed_for_creator_and_community(pool, banned_person_id, community_id, remove)
      .await?;

  create_modlog_entries_for_removed_or_restored_comments(
    pool,
    mod_person_id,
    removed_comment_ids,
    remove,
    reason,
  )
  .await?;

  Ok(())
}

pub async fn purge_user_account(
  person_id: PersonId,
  local_instance_id: InstanceId,
  context: &FastJobContext,
) -> FastJobResult<()> {
  let pool = &mut context.pool();

  // Delete their local images, if they're a local user
  // No need to update avatar and banner, those are handled in Person::delete_account
  delete_local_user_images(person_id, context).await.ok();

  // Comments
  Comment::permadelete_for_creator(pool, person_id)
    .await
    .with_fastjob_type(FastJobErrorType::CouldntUpdateComment)?;

  // Posts
  Post::permadelete_for_creator(pool, person_id)
    .await
    .with_fastjob_type(FastJobErrorType::CouldntUpdatePost)?;

  // Leave communities they mod
  CommunityActions::leave_mod_team_for_all_communities(pool, person_id).await?;

  // Delete the oauth accounts linked to the local user
  if let Ok(local_user) = LocalUserView::read_person(pool, person_id).await {
    OAuthAccount::delete_user_accounts(pool, local_user.local_user.id).await?;
  }

  Person::delete_account(pool, person_id, local_instance_id).await?;

  Ok(())
}

pub fn generate_followers_url(ap_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{ap_id}/followers"))?.into())
}

pub fn generate_inbox_url() -> FastJobResult<DbUrl> {
  let url = format!("{}/inbox", SETTINGS.get_protocol_and_hostname());
  Ok(Url::parse(&url)?.into())
}

pub fn generate_outbox_url(ap_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{ap_id}/outbox"))?.into())
}

pub fn generate_featured_url(ap_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{ap_id}/featured"))?.into())
}

pub fn generate_moderators_url(community_id: &DbUrl) -> FastJobResult<DbUrl> {
  Ok(Url::parse(&format!("{community_id}/moderators"))?.into())
}

/// Ensure that ban/block expiry is in valid range. If its in past, throw error. If its more
/// than 10 years in future, convert to permanent ban. Otherwise return the same value.
pub fn check_expire_time(expires_unix_opt: Option<i64>) -> FastJobResult<Option<DateTime<Utc>>> {
  if let Some(expires_unix) = expires_unix_opt {
    let expires = Utc
      .timestamp_opt(expires_unix, 0)
      .single()
      .ok_or(FastJobErrorType::InvalidUnixTime)?;

    limit_expire_time(expires)
  } else {
    Ok(None)
  }
}

fn limit_expire_time(expires: DateTime<Utc>) -> FastJobResult<Option<DateTime<Utc>>> {
  const MAX_BAN_TERM: Days = Days::new(10 * 365);

  if expires < Local::now() {
    Err(FastJobErrorType::BanExpirationInPast)?
  } else if expires > Local::now() + MAX_BAN_TERM {
    Ok(None)
  } else {
    Ok(Some(expires))
  }
}

pub fn check_conflicting_like_filters(
  liked_only: Option<bool>,
  disliked_only: Option<bool>,
) -> FastJobResult<()> {
  if liked_only.unwrap_or_default() && disliked_only.unwrap_or_default() {
    Err(FastJobErrorType::ContradictingFilters)?
  } else {
    Ok(())
  }
}

pub async fn process_markdown(
  text: &str,
  slur_regex: &Regex,
  url_blocklist: &RegexSet,
  context: &FastJobContext,
) -> FastJobResult<String> {
  let text = remove_slurs(text, slur_regex);
  let text = clean_urls_in_text(&text);

  markdown_check_for_blocked_urls(&text, url_blocklist)?;

  if context.settings().pictrs()?.image_mode == PictrsImageMode::ProxyAllImages {
    let (text, links) = markdown_rewrite_image_links(text);
    RemoteImage::create(&mut context.pool(), links.clone()).await?;

    // Create images and image detail rows
    for link in links {
      // Insert image details for the remote image
      let details_res = fetch_pictrs_proxied_image_details(&link, context).await;
      if let Ok(details) = details_res {
        let proxied = build_proxied_image_url(&link, false, context)?;
        let details_form = details.build_image_details_form(&proxied);
        ImageDetails::create(&mut context.pool(), &details_form).await?;
      }
    }
    Ok(text)
  } else {
    Ok(text)
  }
}

pub async fn process_markdown_opt(
  text: &Option<String>,
  slur_regex: &Regex,
  url_blocklist: &RegexSet,
  context: &FastJobContext,
) -> FastJobResult<Option<String>> {
  match text {
    Some(t) => process_markdown(t, slur_regex, url_blocklist, context)
      .await
      .map(Some),
    None => Ok(None),
  }
}

/// A wrapper for `proxy_image_link` for use in tests.
///
/// The parameter `force_image_proxy` is the config value of `pictrs.image_proxy`. Its necessary to
/// pass as separate parameter so it can be changed in tests.
async fn proxy_image_link_internal(
  link: Url,
  image_mode: PictrsImageMode,
  is_thumbnail: bool,
  context: &FastJobContext,
) -> FastJobResult<DbUrl> {
  // Dont rewrite links pointing to local domain.
  if link.domain() == Some(&context.settings().hostname) {
    Ok(link.into())
  } else if image_mode == PictrsImageMode::ProxyAllImages {
    RemoteImage::create(&mut context.pool(), vec![link.clone()]).await?;

    let proxied = build_proxied_image_url(&link, is_thumbnail, context)?;
    // This should fail softly, since pictrs might not even be running
    let details_res = fetch_pictrs_proxied_image_details(&link, context).await;

    if let Ok(details) = details_res {
      let details_form = details.build_image_details_form(&proxied);
      ImageDetails::create(&mut context.pool(), &details_form).await?;
    };

    Ok(proxied.into())
  } else {
    Ok(link.into())
  }
}

/// Rewrite a link to go through `/api/v4/image_proxy` endpoint. This is only for remote urls and
/// if image_proxy setting is enabled.
pub async fn proxy_image_link(
  link: Url,
  is_thumbnail: bool,
  context: &FastJobContext,
) -> FastJobResult<DbUrl> {
  proxy_image_link_internal(
    link,
    context.settings().pictrs()?.image_mode,
    is_thumbnail,
    context,
  )
  .await
}

pub async fn proxy_image_link_opt_apub(
  link: Option<Url>,
  context: &FastJobContext,
) -> FastJobResult<Option<DbUrl>> {
  if let Some(l) = link {
    proxy_image_link(l, false, context).await.map(Some)
  } else {
    Ok(None)
  }
}

fn build_proxied_image_url(
  link: &Url,
  is_thumbnail: bool,
  context: &FastJobContext,
) -> FastJobResult<Url> {
  let mut url = format!(
    "{}/api/v4/image/proxy?url={}",
    context.settings().get_protocol_and_hostname(),
    encode(link.as_str()),
  );
  if is_thumbnail {
    url = format!(
      "{url}&max_size={}",
      context.settings().pictrs()?.max_thumbnail_size
    );
  }
  Ok(Url::parse(&url)?)
}

pub async fn local_user_view_from_jwt(
  jwt: &str,
  context: &FastJobContext,
) -> FastJobResult<(LocalUserView, String)> {
  let (local_user_id, session) = Claims::validate(jwt, context)
    .await
    .with_fastjob_type(FastJobErrorType::NotLoggedIn)?;
  let local_user_view = LocalUserView::read(&mut context.pool(), local_user_id).await?;
  check_local_user_deleted(&local_user_view)?;

  Ok((local_user_view, session))
}

pub fn read_auth_token(req: &HttpRequest) -> FastJobResult<Option<String>> {
  // Try reading jwt from auth header
  if let Ok(header) = Authorization::<Bearer>::parse(req) {
    Ok(Some(header.as_ref().token().to_string()))
  }
  // If that fails, try to read from cookie
  else if let Some(cookie) = &req.cookie(AUTH_COOKIE_NAME) {
    Ok(Some(cookie.value().to_string()))
  }
  // Otherwise, there's no auth
  else {
    Ok(None)
  }
}

pub fn send_webmention(post: Post, community: &Community) {
  if let Some(url) = post.url.clone() {
    if community.visibility.can_view_without_login() {
      spawn_try_task(async move {
        let mut webmention = Webmention::new::<Url>(post.ap_id.clone().into(), url.clone().into())?;
        webmention.set_checked(true);
        match webmention
          .send()
          .instrument(tracing::info_span!("Sending webmention"))
          .await
        {
          Err(WebmentionError::NoEndpointDiscovered(_)) => Ok(()),
          Ok(_) => Ok(()),
          Err(e) => Err(e).with_fastjob_type(FastJobErrorType::CouldntSendWebmention),
        }
      });
    }
  };
}


/// Extracts the username from an multilang address by taking the part before the @ symbol
pub fn extract_username(email: String) -> Option<String> {
  email.split('@').next().map(|s| s.to_string())
}

/// Generates a unique username from an multilang address
///
/// If the extracted username is available, it will be used.
/// If not, it will try adding random numbers (instead of sequential) until it finds
/// an available username or reaches the maximum attempt limit.
pub async fn generate_unique_username(pool: &mut DbPool<'_>, email: String) -> FastJobResult<String> {
  // Extract username from multilang
  let mut base_username = extract_username(email).unwrap_or_else(|| "user".to_string());

  // Ensure the base username only contains valid characters
  let valid_chars_regex = Regex::new(r"^[a-zA-Z0-9]+$")?;

  // Clean the username to only include valid characters
  if !valid_chars_regex.is_match(&base_username) {
    base_username = base_username
      .chars()
      .filter(|c| c.is_alphanumeric() || *c == '_')
      .collect::<String>();

    // If filtering removed all characters, use "user" as fallback
    if base_username.is_empty() {
      base_username = "user".to_string();
    }
  }

  // First check if the base username is available
  if Person::check_username_taken(pool, &base_username).await.is_err() {
    // Username is already taken, try with random numbers
    let mut rng = rand::rng();
    let mut attempts = 0;
    const MAX_ATTEMPTS: usize = 15;

    while attempts < MAX_ATTEMPTS {
      // Generate a random 4-digit number
      let random_num = rng.random_range(1000..10000);
      let try_username = format!("{}{}", base_username, random_num);

      if Person::check_username_taken(pool, &try_username).await.is_ok() {
        return Ok(try_username);
      }

      attempts += 1;
    }

    // If all attempts failed, generate a completely random username
    let random_num = rng.random_range(10000..100000);
    return Ok(format!("user{}", random_num));
  }

  // Base username is available
  Ok(base_username)
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use lemmy_utils::utils::validation::password_length_check;

  #[test]
  #[rustfmt::skip]
  fn password_length() {
    assert!(password_length_check("Õ¼¾°3yË,o¸ãtÌÈú|ÇÁÙAøüÒI©·¤(T]/ð>æºWæ[C¤bªWöaÃÎñ·{=û³&§½K/c").is_ok());
    assert!(password_length_check("1234567890").is_ok());
    assert!(password_length_check("short").is_err());
    assert!(password_length_check("looooooooooooooooooooooooooooooooooooooooooooooooooooooooooong").is_err());
  }

  #[test]
  fn honeypot() {
    assert!(honeypot_check(&None).is_ok());
    assert!(honeypot_check(&Some(String::new())).is_ok());
    assert!(honeypot_check(&Some("1".to_string())).is_err());
    assert!(honeypot_check(&Some("message".to_string())).is_err());
  }

  #[test]
  fn test_limit_ban_term() -> FastJobResult<()> {
    // Ban expires in past, should throw error
    assert!(limit_expire_time(Utc::now() - Days::new(5)).is_err());

    // Legitimate ban term, return same value
    let fourteen_days = Utc::now() + Days::new(14);
    assert_eq!(limit_expire_time(fourteen_days)?, Some(fourteen_days));
    let nine_years = Utc::now() + Days::new(365 * 9);
    assert_eq!(limit_expire_time(nine_years)?, Some(nine_years));

    // Too long ban term, changes to None (permanent ban)
    assert_eq!(limit_expire_time(Utc::now() + Days::new(365 * 11))?, None);

    Ok(())
  }
}