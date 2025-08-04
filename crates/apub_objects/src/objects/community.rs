use crate::{
  objects::instance::fetch_instance_actor_for_object,
  protocol::group::Group,
  utils::{
    functions::{
      check_apub_id_valid_with_strictness,
      community_visibility,
      read_from_string_or_source_opt,
      GetActorType,
    },
    markdown_links::markdown_rewrite_remote_links_opt,
    protocol::{AttributedTo, LanguageTag, Source},
  },
};

use chrono::{DateTime, Utc};
use lemmy_api_utils::{
  context::FastJobContext,
  utils::{
    generate_featured_url,
    generate_moderators_url,
    generate_outbox_url,
    get_url_blocklist,
    process_markdown_opt,
    proxy_image_link_opt_apub,
    slur_regex,
  },
};
use lemmy_db_schema::{
  sensitive::SensitiveString,
  source::{
    actor_language::CommunityLanguage,
    community::{Community, CommunityInsertForm, CommunityUpdateForm},
  },
  traits::{ApubActor, Crud},
};
use lemmy_db_schema_file::enums::{ActorType, CommunityVisibility};
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::{FastJobError, FastJobResult},
  utils::{
    markdown::markdown_to_html,
    slurs::{check_slurs, check_slurs_opt},
    validation::truncate_description,
  },
};
use once_cell::sync::OnceCell;
use std::ops::Deref;
use actix_web::web::Data;
use url::Url;
use lemmy_api_utils::utils::check_nsfw_allowed;
use crate::fake_trait::{Actor, Object};

#[allow(clippy::type_complexity)]
pub static FETCH_COMMUNITY_COLLECTIONS: OnceCell<
  fn(ApubCommunity, Group, Data<FastJobContext>) -> (),
> = OnceCell::new();

#[derive(Clone, Debug)]
pub struct ApubCommunity(Community);

impl Deref for ApubCommunity {
  type Target = Community;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Community> for ApubCommunity {
  fn from(c: Community) -> Self {
    ApubCommunity(c)
  }
}


impl Object for ApubCommunity {
  type DataType = FastJobContext;
  type Kind = Group;
  type Error = FastJobError;

  fn id(&self) -> &Url {
    self.ap_id.inner()
  }

}

impl Actor for ApubCommunity {
  fn public_key_pem(&self) -> &str {
    todo!()
  }

  fn private_key_pem(&self) -> Option<String> {
    todo!()
  }

  fn inbox(&self) -> Url {
    self.inbox_url.clone().into()
  }

  fn shared_inbox(&self) -> Option<Url> {
    None
  }
}

impl GetActorType for ApubCommunity {
  fn actor_type(&self) -> ActorType {
    ActorType::Community
  }
}
