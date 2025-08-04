
use crate::fake_trait::Object;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::comment::Comment;
use lemmy_utils::error::FastJobError;
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubComment(pub Comment);

impl Deref for ApubComment {
  type Target = Comment;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Comment> for ApubComment {
  fn from(c: Comment) -> Self {
    ApubComment(c)
  }
}


impl Object for ApubComment {
  type DataType = FastJobContext;
  type Kind = String;
  type Error = FastJobError;

  fn id(&self) -> &Url {
    self.ap_id.inner()
  }

}

