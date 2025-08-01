
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityInsertForm, CommunityUpdateForm},
  },
};

use std::ops::Deref;
use url::Url;


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

