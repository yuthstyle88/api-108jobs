use super::check_community_visibility_allowed;
use actix_web::web::{Data, Json};
use lemmy_api_utils::{
  build_response::build_community_response,
  context::FastJobContext,
  utils::{
    check_self_promotion_allowed
    ,
    generate_inbox_url,
    get_url_blocklist,
    is_admin,
    process_markdown_opt,
    slur_regex,
  },
};
use lemmy_db_schema::{
  source::{
    actor_language::{CommunityLanguage, LocalUserLanguage, SiteLanguage},
    community::{
      Community

      ,
      CommunityInsertForm
      ,
    },
  },
  traits::Crud,
};
use lemmy_db_views_community::api::{CommunityResponse, CreateCommunity, CreateCommunityRequest};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::{error::{FastJobErrorType, FastJobResult}, utils::{
  slurs::check_slurs,
  validation::{
    is_valid_body_field,
    site_or_community_description_length_check,
  },
}, MAX_COMMUNITY_DEPTH_LIMIT};

pub async fn create_community(
  data: Json<CreateCommunityRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CommunityResponse>> {
  is_admin(&local_user_view)?;
  
  let data: CreateCommunity = data.into_inner().try_into()?;
  Community::check_community_slug_taken(&mut context.pool(), &data.slug).await?;

  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_site = site_view.local_site;

  if local_site.community_creation_admin_only && is_admin(&local_user_view).is_err() {
    Err(FastJobErrorType::OnlyAdminsCanCreateCommunities)?
  }

  check_self_promotion_allowed(data.self_promotion, Some(&local_site))?;
  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  check_slurs(&data.name, &slur_regex)?;
  check_slurs(&data.title, &slur_regex)?;
  let sidebar = process_markdown_opt(&data.sidebar, &slur_regex, &url_blocklist, &context).await?;

  // Ensure that the sidebar has fewer than the max num characters...
  if let Some(sidebar) = &sidebar {
    is_valid_body_field(sidebar, false)?;
  }

  let description = data.description.clone();
  if let Some(desc) = &description {
    site_or_community_description_length_check(desc)?;
    check_slurs(desc, &slur_regex)?;
  }

  if let Some(desc) = &data.description {
    is_valid_body_field(desc, false)?;
  }

  check_community_visibility_allowed(data.visibility, &local_user_view)?;

  // Double check for duplicate community actor_ids

  let community_form = CommunityInsertForm {
    sidebar,
    description,
    self_promotion: data.self_promotion,
    inbox_url: Some(generate_inbox_url()?),
    posting_restricted_to_mods: data.posting_restricted_to_mods,
    visibility: data.visibility,
    ..CommunityInsertForm::new(
      site_view.site.instance_id,
      data.name.clone(),
      data.title.clone(),
      data.slug.clone(),
    )
  };

  let parent_opt = if let Some(parent_id) = data.parent_id {
    Community::read(&mut context.pool(), parent_id).await.ok()
  } else {
    None
  };

  if let Some(parent) = parent_opt.as_ref() {
    check_community_depth(parent)?;
  }

  let inserted_community = Community::create_sub(&mut context.pool(), &community_form, parent_opt.as_ref()).await?;
  let community_id = inserted_community.id;

  // Update the discussion_languages if that's provided
  let site_languages = SiteLanguage::read_local_raw(&mut context.pool()).await?;
  let languages = if let Some(languages) = data.discussion_languages.clone() {
    // check that community languages are a subset of site languages
    // https://stackoverflow.com/a/64227550
    let is_subset = languages.iter().all(|item| site_languages.contains(item));
    if !is_subset {
      Err(FastJobErrorType::LanguageNotAllowed)?
    }
    languages
  } else {
    // Copy languages from creator
    LocalUserLanguage::read(&mut context.pool(), local_user_view.local_user.id)
      .await?
      .into_iter()
      .filter(|l| site_languages.contains(l))
      .collect()
  };
  CommunityLanguage::update(&mut context.pool(), languages, community_id).await?;

  build_community_response(&context, local_user_view, community_id).await
}

pub fn check_community_depth(community: &Community) -> FastJobResult<()> {
  let path = &community.path.0;
  let length = path.split('.').count();
  if length > MAX_COMMUNITY_DEPTH_LIMIT {
    Err(FastJobErrorType::MaxCommunityDepthReached)?
  } else {
    Ok(())
  }
}