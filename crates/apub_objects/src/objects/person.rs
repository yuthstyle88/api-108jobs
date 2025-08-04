use crate::{
  protocol::person::Person,
  utils::functions::GetActorType,
};

use crate::fake_trait::{Actor, Object};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::{
  source::person::Person as DbPerson,
  traits::{ApubActor, Crud},
};
use lemmy_db_schema_file::enums::ActorType;
use lemmy_utils::error::{FastJobError, FastJobResult};
use std::ops::Deref;
use url::Url;

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
  use crate::objects::instance::ApubSite;
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
