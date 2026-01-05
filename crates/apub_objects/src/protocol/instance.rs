use crate::utils::protocol::{LanguageTag, Source};

use crate::fake_trait::PublicKey;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
  #[serde(rename = "type")]
  pub(crate) kind: String,
  pub(crate) id: String,
  /// site name
  pub(crate) name: String,
  /// instance domain, necessary for mastodon authorized fetch
  pub(crate) preferred_username: Option<String>,
  pub(crate) inbox: Url,
  /// mandatory field in activitypub, app_108jobs currently serves an empty outbox
  pub(crate) outbox: Url,
  pub(crate) public_key: PublicKey,

  // sidebar
  pub(crate) content: Option<String>,
  pub(crate) source: Option<Source>,
  pub(crate) media_type: Option<String>,
  // short instance description
  pub(crate) summary: Option<String>,
  /// instance icon
  pub(crate) icon: Option<String>,
  /// instance banner
  pub(crate) image: Option<String>,
  #[serde(default)]
  pub(crate) language: Vec<LanguageTag>,
  /// nonstandard field
  pub(crate) content_warning: Option<String>,
  pub(crate) published: DateTime<Utc>,
  pub(crate) updated: Option<DateTime<Utc>>,
}
