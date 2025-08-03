use activitypub_federation::config::Data;
use super::check_community_visibility_allowed;
use actix_web::web::{Json};
use chrono::Utc;
use lemmy_api_utils::utils::{check_community_deleted_removed, is_admin};
use lemmy_api_utils::{
  build_response::build_community_response,
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_self_promotion_allowed, get_url_blocklist, process_markdown_opt, slur_regex},
};
use lemmy_db_schema::{
  source::{
    actor_language::{CommunityLanguage, SiteLanguage},
    community::{Community, CommunityUpdateForm}
    ,
  },
  traits::Crud,
  utils::diesel_string_update,
};
use lemmy_db_views_community::api::{CommunityResponse, EditCommunity};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::{FastJobErrorType, FastJobResult},
  utils::{slurs::check_slurs_opt, validation::is_valid_body_field},
};

pub async fn update_community(
  data: Json<EditCommunity>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CommunityResponse>> {
  is_admin(&local_user_view)?;

  if let Some(ref slug) = data.slug {
    Community::check_community_slug_taken(&mut context.pool(), slug).await?;
  }
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;

  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  check_slurs_opt(&data.title, &slur_regex)?;
  check_self_promotion_allowed(data.self_promotion, Some(&local_site))?;

  let sidebar = diesel_string_update(
    process_markdown_opt(&data.sidebar, &slur_regex, &url_blocklist, &context)
      .await?
      .as_deref(),
  );

  if let Some(Some(sidebar)) = &sidebar {
    is_valid_body_field(sidebar, false)?;
  }

  check_community_visibility_allowed(data.visibility, &local_user_view)?;
  let description = diesel_string_update(data.description.as_deref());

  let old_community = Community::read(&mut context.pool(), data.community_id).await?;

  check_community_deleted_removed(&old_community)?;

  let community_id = data.community_id;
  if let Some(languages) = data.discussion_languages.clone() {
    let site_languages = SiteLanguage::read_local_raw(&mut context.pool()).await?;
    // check that community languages are a subset of site languages
    // https://stackoverflow.com/a/64227550
    let is_subset = languages.iter().all(|item| site_languages.contains(item));
    if !is_subset {
      Err(FastJobErrorType::LanguageNotAllowed)?
    }
    CommunityLanguage::update(&mut context.pool(), languages, community_id).await?;
  }

  let community_form = CommunityUpdateForm {
    name: data.name.clone(),
    title: data.title.clone(),
    sidebar,
    description,
    self_promotion: data.self_promotion,
    posting_restricted_to_mods: data.posting_restricted_to_mods,
    visibility: data.visibility,
    updated_at: Some(Some(Utc::now())),
    slug: data.slug.clone(),
    is_new: data.is_new,
    ..Default::default()
  };

  let community_id = data.community_id;
  let community = Community::update(&mut context.pool(), community_id, &community_form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::UpdateCommunity(local_user_view.person.clone(), community),
    &context,
  )?;

  build_community_response(&context, local_user_view, community_id).await
}
