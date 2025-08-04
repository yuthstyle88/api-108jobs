use crate::{
  protocol::page::{
    Attachment,
    Hashtag,
    HashtagType::{self},
    Page,
    PageType,
  },
  utils::{
    functions::{
      check_apub_id_valid_with_strictness,
      generate_to,
      read_from_string_or_source_opt,
      verify_person_in_community,
    },
    markdown_links::{markdown_rewrite_remote_links_opt, to_local_url},
    protocol::{AttributedTo, InCommunity, LanguageTag, Source},
  },
};
use anyhow::anyhow;
use chrono::Utc;
use html2text::{from_read_with_decorator, render::TrivialDecorator};
use lemmy_api_utils::{
  context::FastJobContext,
  plugins::{plugin_hook_after},
  request::generate_post_link_metadata,
  utils::{get_url_blocklist, process_markdown_opt, slur_regex},
};
use lemmy_db_schema::{
  source::{
    community::Community,
    person::Person,
    post::{Post, PostInsertForm, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::{FastJobError, FastJobResult},
  spawn_try_task,
  utils::{
    markdown::markdown_to_html,
    slurs::check_slurs_opt,
    validation::{is_url_blocked, is_valid_url},
  },
};
use std::ops::Deref;
use actix_web::web::Data;
use stringreader::StringReader;
use url::Url;
use crate::fake_trait::Object;

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

