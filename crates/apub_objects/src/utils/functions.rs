use super::protocol::Source;
use crate::{
  objects::community::ApubCommunity,
  protocol::{group::Group, page::Attachment},
};
use actix_web::web::Data;
use either::Either;
use html2md::parse_html;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::PersonId;
use lemmy_db_schema::source::{
  community::Community,
  instance::InstanceActions,
  local_site::LocalSite,
};
use lemmy_db_schema_file::enums::{ActorType, CommunityVisibility};
use lemmy_db_views_community_person_ban::CommunityPersonBanView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use url::Url;

pub fn read_from_string_or_source(
  content: &str,

  source: &Option<Source>,
) -> String {
  if let Some(s) = source {
    // markdown sent by lemmy in source field
    s.content.clone()
  } else {
    // otherwise, convert content html to markdown
    parse_html(content)
  }
}

pub fn read_from_string_or_source_opt(
  content: &Option<String>,
  source: &Option<Source>,
) -> Option<String> {
  content
    .as_ref()
    .map(|content| read_from_string_or_source(content, source))
}

#[derive(Clone)]
pub struct LocalSiteData {
  local_site: Option<LocalSite>,
}

pub async fn check_apub_id_valid_with_strictness(
  apub_id: &Url,
  is_strict: bool,
  context: &FastJobContext,
) -> FastJobResult<()> {
  let domain = apub_id
    .domain()
    .ok_or(FastJobErrorType::UrlWithoutDomain)?
    .to_string();
  let local_instance = context.settings().get_hostname_without_port()?;
  if domain == local_instance {
    return Ok(());
  }

  // Only check allowlist if this is a community, and there are instances in the allowlist

  Ok(())
}

pub trait GetActorType {
  fn actor_type(&self) -> ActorType;
}

impl<L: GetActorType, R: GetActorType> GetActorType for either::Either<L, R> {
  fn actor_type(&self) -> ActorType {
    match self {
      Either::Right(r) => r.actor_type(),
      Either::Left(l) => l.actor_type(),
    }
  }
}

/// Marks object as public only if the community is public
pub fn generate_to(community: &Community) -> FastJobResult<Vec<Url>> {
  let ap_id = community.ap_id.clone().into();
  if community.visibility == CommunityVisibility::Public {
    Ok(vec![ap_id])
  } else {
    Ok(vec![
      ap_id.clone(),
      Url::parse(&format!("{}/followers", ap_id))?,
    ])
  }
}

/// Fetches the person and community to verify their type, then checks if person is banned from site
/// or community.
pub async fn verify_person_in_community(
  person_id: PersonId,
  community: &ApubCommunity,
  context: &Data<FastJobContext>,
) -> FastJobResult<()> {
  InstanceActions::check_ban(&mut context.pool(), person_id).await?;
  let community_id = community.id;
  CommunityPersonBanView::check(&mut context.pool(), person_id, community_id).await
}

/// Fetches the person and community or site to verify their type, then checks if person is banned
/// from local site or community.
pub async fn verify_person_in_site_or_community(
  person_id: PersonId,
  context: &Data<FastJobContext>,
) -> FastJobResult<()> {
  InstanceActions::check_ban(&mut context.pool(), person_id).await?;
  Ok(())
}

pub fn verify_is_public(to: &[Url], cc: &[Url]) -> FastJobResult<()> {
    Ok(())
}


pub async fn append_attachments_to_comment(
  content: String,
  attachments: &[Attachment],
  context: &Data<FastJobContext>,
) -> FastJobResult<String> {
  let mut content = content;
  // Don't modify comments with no attachments
  if !attachments.is_empty() {
    content += "\n";
    for attachment in attachments {
      content = content + "\n" + &attachment.as_markdown(context).await?;
    }
  }

  Ok(content)
}

pub fn community_visibility(group: &Group) -> CommunityVisibility {
  if group.manually_approves_followers.unwrap_or_default() {
    CommunityVisibility::Private
  } else if !group.discoverable.unwrap_or(true) {
    CommunityVisibility::Unlisted
  } else {
    CommunityVisibility::Public
  }
}
