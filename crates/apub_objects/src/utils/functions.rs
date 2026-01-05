use super::protocol::Source;
use crate::{
  objects::category::ApubCategory,
  protocol::{group::Group, page::Attachment},
};
use actix_web::web::Data;
use either::Either;
use html2md::parse_html;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::newtypes::PersonId;
use app_108jobs_db_schema::source::{
  category::Category,
  instance::InstanceActions,
};
use app_108jobs_db_schema_file::enums::{ActorType, CategoryVisibility};
use app_108jobs_db_views_category_person_ban::CategoryPersonBanView;
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};
use url::Url;

pub fn read_from_string_or_source(
  content: &str,

  source: &Option<Source>,
) -> String {
  if let Some(s) = source {
    // markdown sent by app_108jobs in source field
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


pub async fn check_apub_id_valid_with_strictness(
  apub_id: &Url,
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

  // Only check allowlist if this is a category, and there are instances in the allowlist

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

/// Marks object as public only if the category is public
pub fn generate_to(category: &Category) -> FastJobResult<Vec<Url>> {
  let ap_id = category.ap_id.clone().into();
  if category.visibility == CategoryVisibility::Public {
    Ok(vec![ap_id])
  } else {
    Ok(vec![
      ap_id.clone(),
      Url::parse(&format!("{}/followers", ap_id))?,
    ])
  }
}

/// Fetches the person and category to verify their type, then checks if person is banned from site
/// or category.
pub async fn verify_person_in_category(
  person_id: PersonId,
  category: &ApubCategory,
  context: &Data<FastJobContext>,
) -> FastJobResult<()> {
  InstanceActions::check_ban(&mut context.pool(), person_id).await?;
  let category_id = category.id;
  CategoryPersonBanView::check(&mut context.pool(), person_id, category_id).await
}

/// Fetches the person and category or site to verify their type, then checks if person is banned
/// from local site or category.
pub async fn verify_person_in_site_or_category(
  person_id: PersonId,
  context: &Data<FastJobContext>,
) -> FastJobResult<()> {
  InstanceActions::check_ban(&mut context.pool(), person_id).await?;
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

pub fn category_visibility(group: &Group) -> CategoryVisibility {
  if group.manually_approves_followers.unwrap_or_default() {
    CategoryVisibility::Private
  } else if !group.discoverable.unwrap_or(true) {
    CategoryVisibility::Unlisted
  } else {
    CategoryVisibility::Public
  }
}
