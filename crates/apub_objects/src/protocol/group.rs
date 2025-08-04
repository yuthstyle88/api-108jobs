use crate::fake_trait::PublicKey;
use crate::utils::protocol::{AttributedTo, Endpoints, LanguageTag, Source};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fmt::Debug;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Group {
  #[serde(rename = "type")]
  pub(crate) kind: String,
  pub id: String,
  /// username, set at account creation and usually fixed after that
  pub preferred_username: String,
  pub inbox: Url,
  pub followers: Option<Url>,
  pub public_key: PublicKey,

  /// title
  pub name: Option<String>,
  // sidebar
  pub(crate) content: Option<String>,

  pub source: Option<Source>,
  pub(crate) media_type: Option<String>,
  // short instance description
  pub summary: Option<String>,
  pub icon: Option<String>,
  pub image: Option<String>,
  // lemmy extension
  pub sensitive: Option<bool>,
  pub attributed_to: Option<AttributedTo>,
  // lemmy extension
  pub posting_restricted_to_mods: Option<bool>,
  pub outbox: Url,
  pub endpoints: Option<Endpoints>,
  pub featured: Option<Url>,
  #[serde(default)]
  pub(crate) language: Vec<LanguageTag>,
  /// True if this is a private community
  pub(crate) manually_approves_followers: Option<bool>,
  pub published: Option<DateTime<Utc>>,
  pub updated: Option<DateTime<Utc>>,
  /// https://docs.joinmastodon.org/spec/activitypub/#discoverable
  pub(crate) discoverable: Option<bool>,
}
