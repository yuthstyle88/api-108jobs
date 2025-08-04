use crate::{
  utils::{
    functions::{
      append_attachments_to_comment,
      check_apub_id_valid_with_strictness,
      generate_to,
      read_from_string_or_source,
      verify_person_in_community,
    },
    markdown_links::markdown_rewrite_remote_links,
    protocol::{InCommunity, LanguageTag, Source},
  },
};

use chrono::{DateTime, Utc};
use lemmy_api_utils::{
  context::FastJobContext,
  plugins::{plugin_hook_after},
  utils::{
    check_comment_depth,
    check_is_mod_or_admin,
    get_url_blocklist,
    process_markdown,
    slur_regex,
  },
};
use lemmy_db_schema::{
  source::{
    comment::{Comment, CommentInsertForm, CommentUpdateForm},
    community::Community,
    person::Person,
    post::Post,
  },
  traits::Crud,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::{FastJobError, FastJobResult},
  utils::markdown::markdown_to_html,
};
use std::ops::Deref;
use actix_web::web::Data;
use url::Url;
use lemmy_utils::error::FastJobErrorType;
use crate::fake_trait::{Note, Object};

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

