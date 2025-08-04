use crate::objects::{person::ApubPerson};
use lemmy_db_schema::{
  source::{comment::Comment, person::Person, post::Post},
  traits::Crud,
  utils::DbPool,
};
use lemmy_utils::{
  error::{FastJobResult},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MentionOrValue {
  Mention(Mention),
  Value(Value),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Mention {
  pub href: Url,
  name: Option<String>,
}

pub struct MentionsAndAddresses {
  pub ccs: Vec<Url>,
  pub tags: Vec<MentionOrValue>,
}

/// This takes a comment, and builds a list of to_addresses, inboxes,
/// and mention tags, so they know where to be sent to.
/// Addresses are the persons / addresses that go in the cc field.


/// Returns the apub ID of the person this comment is responding to. Meaning, in case this is a
/// top-level comment, the creator of the post, otherwise the creator of the parent comment.
async fn get_comment_parent_creator(
  pool: &mut DbPool<'_>,
  comment: &Comment,
) -> FastJobResult<ApubPerson> {
  let parent_creator_id = if let Some(parent_comment_id) = comment.parent_comment_id() {
    let parent_comment = Comment::read(pool, parent_comment_id).await?;
    parent_comment.creator_id
  } else {
    let parent_post_id = comment.post_id;
    let parent_post = Post::read(pool, parent_post_id).await?;
    parent_post.creator_id
  };
  Ok(Person::read(pool, parent_creator_id).await?.into())
}
