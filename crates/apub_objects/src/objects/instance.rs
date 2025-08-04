use crate::{
  protocol::instance::Instance,
  utils::functions::GetActorType,
};

use crate::fake_trait::{Actor, Object};
use actix_web::web::Data;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::{
  newtypes::InstanceId,
  sensitive::SensitiveString,
  source::site::Site
  ,
};
use lemmy_db_schema_file::enums::ActorType;
use lemmy_utils::error::{FastJobError, FastJobResult};
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubSite(pub Site);

impl Deref for ApubSite {
  type Target = Site;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Site> for ApubSite {
  fn from(s: Site) -> Self {
    ApubSite(s)
  }
}

impl Object for ApubSite {
  type DataType = FastJobContext;
  type Kind = Instance;
  type Error = FastJobError;

  fn id(&self) -> &Url {
    self.ap_id.inner()
  }

}

impl Actor for ApubSite {
  fn public_key_pem(&self) -> &str {
    &self.public_key
  }

  fn private_key_pem(&self) -> Option<String> {
    self.private_key.clone().map(SensitiveString::into_inner)
  }

  fn inbox(&self) -> Url {
    self.inbox_url.clone().into()
  }
}
impl GetActorType for ApubSite {
  fn actor_type(&self) -> ActorType {
    ActorType::Site
  }
}

/// Try to fetch the instance actor (to make things like instance rules available).
pub(crate) async fn fetch_instance_actor_for_object<T: Into<Url> + Clone>(
  object_id: &T,
  context: &Data<FastJobContext>,
) -> FastJobResult<InstanceId> {

  todo!()
}

