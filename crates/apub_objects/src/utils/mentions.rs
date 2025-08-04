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
