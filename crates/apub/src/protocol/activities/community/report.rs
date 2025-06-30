use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::FlagType,
  protocol::helpers::deserialize_one,
};
use either::Either;
use lemmy_api_utils::context::FastJobContext;
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, instance::ApubSite, person::ApubPerson, ReportableObjects},
  utils::protocol::InCommunity,
};
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one")]
  pub(crate) to: [ObjectId<Either<ApubSite, ApubCommunity>>; 1],
  pub(crate) object: ReportObject,
  /// Report reason as sent by Lemmy
  pub(crate) summary: Option<String>,
  /// Report reason as sent by Mastodon
  pub(crate) content: Option<String>,
  #[serde(rename = "type")]
  pub(crate) kind: FlagType,
  pub(crate) id: Url,
}

impl Report {
  pub fn reason(&self) -> FastJobResult<String> {
    self
      .summary
      .clone()
      .or(self.content.clone())
      .ok_or(FastJobErrorType::NotFound.into())
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum ReportObject {
  FastJob(ObjectId<ReportableObjects>),
  /// Mastodon sends an array containing user id and one or more post ids
  Mastodon(Vec<Url>),
}

impl ReportObject {
  pub(crate) async fn dereference(
    &self,
    context: &Data<FastJobContext>,
  ) -> FastJobResult<ReportableObjects> {
    match self {
      ReportObject::FastJob(l) => l.dereference(context).await,
      ReportObject::Mastodon(objects) => {
        for o in objects {
          // Find the first reported item which can be dereferenced as post or comment (Lemmy can
          // only handle one item per report).
          let deref = ObjectId::from(o.clone()).dereference(context).await;
          if deref.is_ok() {
            return deref;
          }
        }
        Err(FastJobErrorType::NotFound.into())
      }
    }
  }

  pub(crate) async fn object_id(
    &self,
    context: &Data<FastJobContext>,
  ) -> FastJobResult<ObjectId<ReportableObjects>> {
    match self {
      ReportObject::FastJob(l) => Ok(l.clone()),
      ReportObject::Mastodon(objects) => {
        for o in objects {
          // Same logic as above, but return the ID and not the object itself.
          let deref = ObjectId::<ReportableObjects>::from(o.clone())
            .dereference(context)
            .await;
          if deref.is_ok() {
            return Ok(o.clone().into());
          }
        }
        Err(FastJobErrorType::NotFound.into())
      }
    }
  }
}

impl InCommunity for Report {
  async fn community(&self, context: &Data<FastJobContext>) -> FastJobResult<ApubCommunity> {
    match self.to[0].dereference(context).await? {
      Either::Left(_) => Err(FastJobErrorType::NotFound.into()),
      Either::Right(c) => Ok(c),
    }
  }
}
