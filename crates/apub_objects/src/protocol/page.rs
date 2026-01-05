use crate::utils::protocol::{
  AttributedTo,
  LanguageTag
  ,
  Source,
};
use actix_web::web::Data;

use chrono::{DateTime, Utc};
use app_108jobs_api_utils::{context::FastJobContext, utils::proxy_image_link};
use app_108jobs_utils::error::FastJobResult;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum PageType {
  Page,
  Article,
  Note,
  Video,
  Event,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
  #[serde(rename = "type")]
  pub(crate) kind: PageType,
  pub id: String,
  pub(crate) attributed_to: AttributedTo,
  pub(crate) to: Vec<Url>,
  // If there is inReplyTo field this is actually a comment and must not be parsed
  pub(crate) in_reply_to: Option<String>,

  pub(crate) name: Option<String>,
  pub(crate) cc: Vec<Url>,
  pub(crate) content: Option<String>,
  pub(crate) media_type: Option<String>,
  pub(crate) source: Option<Source>,
  /// most software uses array type for attachment field, so we do the same. nevertheless, we only
  /// use the first item
  #[serde(default)]
  pub(crate) attachment: Vec<Attachment>,
  pub(crate) image: Option<String>,
  pub(crate) sensitive: Option<bool>,
  pub(crate) published: Option<DateTime<Utc>>,
  pub(crate) updated: Option<DateTime<Utc>>,
  pub(crate) language: Option<LanguageTag>,
  pub(crate) tag: Vec<Hashtag>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
  href: Url,
  media_type: Option<String>,
  r#type: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
  #[serde(rename = "type")]
  kind: String,
  url: Url,
  /// Used for alt_text
  name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
  #[serde(rename = "type")]
  kind: String,
  url: Url,
  media_type: Option<String>,
  /// Used for alt_text
  name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Attachment {
  Link(Link),
  Image(Image),
  Document(Document),
}

impl Attachment {

  pub(crate) async fn as_markdown(&self, context: &Data<FastJobContext>) -> FastJobResult<String> {
    let (url, name, media_type) = match self {
      Attachment::Image(i) => (i.url.clone(), i.name.clone(), Some(String::from("image"))),
      Attachment::Document(d) => (d.url.clone(), d.name.clone(), d.media_type.clone()),
      Attachment::Link(l) => (l.href.clone(), None, l.media_type.clone()),
    };

    let is_image =
      media_type.is_some_and(|media| media.starts_with("video") || media.starts_with("image"));
    // Markdown images can't have linebreaks in them, so to prevent creating
    // broken image embeds, replace them with spaces
    let name = name.map(|n| n.split_whitespace().collect::<Vec<_>>().join(" "));

    if is_image {
      let url = proxy_image_link(url, false, context).await?;
      Ok(format!("![{}]({url})", name.unwrap_or_default()))
    } else {
      Ok(format!("[{url}]({url})"))
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Hashtag {
  pub(crate) href: Url,
  pub(crate) name: String,
  #[serde(rename = "type")]
  pub(crate) kind: HashtagType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum HashtagType {
  Hashtag,
}





/// Only allows deserialization if the field is missing or null. If it is present, throws an error.
pub fn deserialize_not_present<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  let result: Option<String> = Deserialize::deserialize(deserializer)?;
  match result {
    None => Ok(None),
    Some(_) => Err(D::Error::custom("Post must not have inReplyTo property")),
  }
}


