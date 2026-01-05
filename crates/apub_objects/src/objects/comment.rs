
use crate::fake_trait::Object;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::source::comment::Comment;
use app_108jobs_utils::error::FastJobError;
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

