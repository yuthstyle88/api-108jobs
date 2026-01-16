use crate::{
  protocol::group::Group,
  utils::functions::GetActorType,
};

use crate::fake_trait::{Actor, Object};
use actix_web::web::Data;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::source::category::Category;
use app_108jobs_db_schema_file::enums::ActorType;
use app_108jobs_utils::error::FastJobError;
use once_cell::sync::OnceCell;
use std::ops::Deref;
use url::Url;

#[allow(clippy::type_complexity)]
pub static FETCH_CATEGORY_COLLECTIONS: OnceCell<
  fn(ApubCategory, Group, Data<FastJobContext>) -> (),
> = OnceCell::new();

#[derive(Clone, Debug)]
pub struct ApubCategory(Category);

impl Deref for ApubCategory {
  type Target = Category;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Category> for ApubCategory {
  fn from(c: Category) -> Self {
    ApubCategory(c)
  }
}


impl Object for ApubCategory {
  type DataType = FastJobContext;
  type Kind = Group;
  type Error = FastJobError;

  fn id(&self) -> &Url {
    self.ap_id.inner()
  }

}

impl Actor for ApubCategory {
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

impl GetActorType for ApubCategory {
  fn actor_type(&self) -> ActorType {
    ActorType::Category
  }
}
