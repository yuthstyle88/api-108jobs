use crate::{
  objects::instance::fetch_instance_actor_for_object,
  protocol::person::{Person, UserTypes},
  utils::{
    functions::{
      check_apub_id_valid_with_strictness,
      read_from_string_or_source_opt,
      GetActorType,
    },
    markdown_links::markdown_rewrite_remote_links_opt,
    protocol::{Source},
  },
};

use chrono::{DateTime, Utc};
use lemmy_api_utils::{
  context::FastJobContext,
  utils::{
    generate_outbox_url,
    get_url_blocklist,
    process_markdown_opt,
    proxy_image_link_opt_apub,
    slur_regex,
  },
};
use lemmy_db_schema::{
  source::person::{Person as DbPerson, PersonInsertForm, PersonUpdateForm},
  traits::{ApubActor, Crud},
};
use lemmy_db_schema_file::enums::ActorType;
use lemmy_utils::{
  error::{FastJobError, FastJobResult},
  utils::{
    markdown::markdown_to_html,
    slurs::{check_slurs, check_slurs_opt},
  },
};
use std::ops::Deref;
use actix_web::web::Data;
use url::Url;
use crate::fake_trait::{Actor, Object};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApubPerson(pub DbPerson);

impl Deref for ApubPerson {
  type Target = DbPerson;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<DbPerson> for ApubPerson {
  fn from(p: DbPerson) -> Self {
    ApubPerson(p)
  }
}

impl Object for ApubPerson {
  type DataType = FastJobContext;
  type Kind = Person;
  type Error = FastJobError;

  fn id(&self) -> &Url {
    self.ap_id.inner()
  }

}

impl Actor for ApubPerson {
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

impl GetActorType for ApubPerson {
  fn actor_type(&self) -> ActorType {
    ActorType::Person
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::{
    objects::instance::ApubSite,
  };
  use lemmy_db_schema::source::site::Site;


  async fn cleanup(
    (person, site): (ApubPerson, ApubSite),
    context: &FastJobContext,
  ) -> FastJobResult<()> {
    DbPerson::delete(&mut context.pool(), person.id).await?;
    Site::delete(&mut context.pool(), site.id).await?;
    Ok(())
  }
}
