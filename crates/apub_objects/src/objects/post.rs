use crate::fake_trait::Object;
use crate::{
  protocol::page::Page,
};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::source::post::Post;
use app_108jobs_utils::error::FastJobError;
use std::ops::Deref;
use url::Url;


#[derive(Clone, Debug, PartialEq)]
pub struct ApubPost(pub Post);

impl Deref for ApubPost {
  type Target = Post;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Post> for ApubPost {
  fn from(p: Post) -> Self {
    ApubPost(p)
  }
}

impl Object for ApubPost {
  type DataType = FastJobContext;
  type Kind = Page;
  type Error = FastJobError;

  fn id(&self) -> &Url {
    self.ap_id.inner()
  }

}

