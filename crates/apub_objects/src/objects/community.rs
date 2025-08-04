use crate::{
  protocol::group::Group,
  utils::functions::GetActorType,
};

use crate::fake_trait::{Actor, Object};
use actix_web::web::Data;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::community::Community;
use lemmy_db_schema_file::enums::ActorType;
use lemmy_utils::error::FastJobError;
use once_cell::sync::OnceCell;
use std::ops::Deref;
use url::Url;

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
