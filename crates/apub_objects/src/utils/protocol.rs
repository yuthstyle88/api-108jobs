use crate::objects::community::ApubCommunity;
use actix_web::web::Data;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::{
  impls::actor_language::UNDETERMINED_ID,
  newtypes::{DbUrl, LanguageId},
  source::language::Language,
  utils::DbPool,
};
use lemmy_utils::error::FastJobResult;
use serde::{Deserialize, Serialize};
use std::{future::Future, ops::Deref};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Source {
  pub(crate) content: String,
}

impl Source {
  pub(crate) fn new(content: String) -> Self {
    Source {
      content,
    }
  }
}

pub trait InCommunity {
  fn community(
    &self,
    context: &Data<FastJobContext>,
  ) -> impl Future<Output = FastJobResult<ApubCommunity>> + Send;
}


#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum AttributedTo {
  Lemmy(PersonOrGroupModerators),
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
      AttributedTo::Lemmy(l) => Some(l.moderators().into()),
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
  pub(crate) fn creator(&self) -> String {
    self.deref().clone().into()
  }

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

impl LanguageTag {
  pub(crate) async fn new_single(
    lang: LanguageId,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<LanguageTag> {
    let lang = Language::read_from_id(pool, lang).await?;

    // undetermined
    if lang.id == UNDETERMINED_ID {
      Ok(LanguageTag::default())
    } else {
      Ok(LanguageTag {
        identifier: lang.code,
        name: lang.name,
      })
    }
  }

  pub(crate) async fn new_multiple(
    lang_ids: Vec<LanguageId>,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Vec<LanguageTag>> {
    let mut langs = Vec::<Language>::new();

    for l in lang_ids {
      langs.push(Language::read_from_id(pool, l).await?);
    }

    let langs = langs
     .into_iter()
     .map(|l| LanguageTag {
       identifier: l.code,
       name: l.name,
     })
     .collect();
    Ok(langs)
  }

  pub(crate) async fn to_language_id_single(
    lang: Self,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<LanguageId> {
    Language::read_id_from_code(pool, &lang.identifier).await
  }

  pub(crate) async fn to_language_id_multiple(
    langs: Vec<Self>,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Vec<LanguageId>> {
    let mut language_ids = Vec::new();

    for l in langs {
      let id = l.identifier;
      language_ids.push(Language::read_id_from_code(pool, &id).await?);
    }

    Ok(language_ids.into_iter().collect())
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

