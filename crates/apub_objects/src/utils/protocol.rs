
use app_108jobs_db_schema::{
  newtypes::{DbUrl},
};
use serde::{Deserialize, Serialize};
use std::{ops::Deref};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Source {
  pub(crate) content: String,
}


#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum AttributedTo {
  App108jobs(PersonOrGroupModerators),
  Peertube(Vec<AttributedToPeertube>),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PersonOrGroupType {
  Person,
  Group,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AttributedToPeertube {
  #[serde(rename = "type")]
  pub kind: PersonOrGroupType,
  pub id: String,
}

impl AttributedTo {
  pub fn url(self) -> Option<DbUrl> {
    match self {
      AttributedTo::App108jobs(l) => Some(l.moderators().into()),
      AttributedTo::Peertube(_) => None,
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PersonOrGroupModerators(Url);

impl Deref for PersonOrGroupModerators {
  type Target = Url;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<DbUrl> for PersonOrGroupModerators {
  fn from(value: DbUrl) -> Self {
    PersonOrGroupModerators(value.into())
  }
}

impl PersonOrGroupModerators {
  pub fn moderators(&self) -> Url {
    self.deref().clone()
  }
}

/// As specified in https://schema.org/Language
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LanguageTag {
  pub(crate) identifier: String,
  pub(crate) name: String,
}

impl Default for LanguageTag {
  fn default() -> Self {
    LanguageTag {
      identifier: "und".to_string(),
      name: "Undetermined".to_string(),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Endpoints {
  pub shared_inbox: Url,
}

pub trait Id {
  fn id(&self) -> &Url;
}

