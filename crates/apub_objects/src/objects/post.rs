use crate::fake_trait::Object;
use crate::{
  protocol::page::Page,
  utils::protocol::InCommunity,
};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::post::Post;
use lemmy_utils::error::FastJobError;
use std::ops::Deref;
use url::Url;

const MAX_TITLE_LENGTH: usize = 200;

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

