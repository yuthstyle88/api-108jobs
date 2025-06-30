use super::send_activity_from_user_or_community_or_multi;
use crate::{
  activities::generate_activity_id,
  protocol::activities::following::{follow::Follow, reject::RejectFollow},
};
use activitypub_federation::{
  config::Data,
  kinds::activity::RejectType,
  protocol::verification::verify_urls_match,
  traits::{ActivityHandler, Actor},
};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::{
  source::{activity::ActivitySendTargets, community::CommunityActions},
  traits::Followable,
};
use lemmy_utils::error::{FastJobError, FastJobResult};
use url::Url;

impl RejectFollow {
  pub async fn send(follow: Follow, context: &Data<FastJobContext>) -> FastJobResult<()> {
    let user_or_community = follow.object.dereference_local(context).await?;
    let person = follow.actor.clone().dereference(context).await?;
    let reject = RejectFollow {
      actor: user_or_community.id().into(),
      to: Some([person.id().into()]),
      object: follow,
      kind: RejectType::Reject,
      id: generate_activity_id(RejectType::Reject, context)?,
    };
    let inbox = ActivitySendTargets::to_inbox(person.shared_inbox_or_inbox());
    send_activity_from_user_or_community_or_multi(context, reject, user_or_community, inbox).await
  }
}

/// Handle rejected follows
#[async_trait::async_trait]
impl ActivityHandler for RejectFollow {
  type DataType = FastJobContext;
  type Error = FastJobError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<FastJobContext>) -> FastJobResult<()> {
    verify_urls_match(self.actor.inner(), self.object.object.inner())?;
    self.object.verify(context).await?;
    if let Some(to) = &self.to {
      verify_urls_match(to[0].inner(), self.object.actor.inner())?;
    }
    Ok(())
  }

  async fn receive(self, context: &Data<FastJobContext>) -> FastJobResult<()> {
    let community = self.actor.dereference(context).await?;
    let person = self.object.actor.dereference(context).await?;

    // remove the follow
    CommunityActions::unfollow(&mut context.pool(), person.id, community.id).await?;

    Ok(())
  }
}
