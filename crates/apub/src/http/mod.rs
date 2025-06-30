use crate::{activity_lists::SharedInboxActivities, fetcher::get_instance_id};
use activitypub_federation::{
  actix_web::{
    inbox::{receive_activity_with_hook, ReceiveActivityHook},
    signing_actor,
  },
  config::Data,
  protocol::context::WithContext,
  traits::{ActivityHandler, Actor},
  FEDERATION_CONTENT_TYPE,
};
use actix_web::{
  http::header::VARY,
  web::{self, Bytes},
  HttpRequest,
  HttpResponse,
};
use lemmy_api_utils::{context::FastJobContext, plugins::plugin_hook_after};
use lemmy_apub_objects::{
  objects::{SiteOrMultiOrCommunityOrUser, UserOrCommunity},
  protocol::tombstone::Tombstone,
};
use lemmy_db_schema::{
  newtypes::DbUrl,
  source::{
    activity::{ReceivedActivity, SentActivity},
    community::Community,
  },
};
use lemmy_db_schema_file::enums::CommunityVisibility;
use lemmy_db_views_community_follower::CommunityFollowerView;
use lemmy_utils::{
  error::{FederationError, FastJobErrorExt, FastJobErrorType, FastJobResult},
  FEDERATION_CONTEXT,
};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, time::Duration};
use tokio::time::timeout;
use tracing::debug;
use url::Url;

mod comment;
mod community;
mod person;
mod post;
pub mod routes;
pub mod site;

const INCOMING_ACTIVITY_TIMEOUT: Duration = Duration::from_secs(9);

pub async fn shared_inbox(
  request: HttpRequest,
  body: Bytes,
  data: Data<FastJobContext>,
) -> FastJobResult<HttpResponse> {
  let receive_fut =
    receive_activity_with_hook::<SharedInboxActivities, UserOrCommunity, FastJobContext>(
      request, body, Dummy, &data,
    );
  // Set a timeout shorter than `REQWEST_TIMEOUT` for processing incoming activities. This is to
  // avoid taking a long time to process an incoming activity when a required data fetch times out.
  // In this case our own instance would timeout and be marked as dead by the sender. Better to
  // consider the activity broken and move on.
  timeout(INCOMING_ACTIVITY_TIMEOUT, receive_fut)
    .await
    .with_fastjob_type(FederationError::InboxTimeout.into())?
}

struct Dummy;

impl ReceiveActivityHook<SharedInboxActivities, UserOrCommunity, FastJobContext> for Dummy {
  async fn hook(
    self,
    activity: &SharedInboxActivities,
    _actor: &UserOrCommunity,
    context: &Data<FastJobContext>,
  ) -> FastJobResult<()> {
    // Store received activities in the database. This ensures that the same activity doesn't get
    // received and processed more than once, which would be a waste of resources.
    debug!("Received activity {}", activity.id().to_string());
    ReceivedActivity::create(&mut context.pool(), &activity.id().clone().into()).await?;

    // This could also take the actor as param, but lifetimes and serde derives are tricky.
    // It is really a before hook, but doesnt allow modifying the data. It could use a
    // separate method so that error in plugin causes activity to be rejected.
    plugin_hook_after("activity_received", activity)?;

    // This method could also be used to check if actor is banned, instead of checking in each
    // activity handler.
    Ok(())
  }
}

/// Convert the data to json and turn it into an HTTP Response with the correct ActivityPub
/// headers.
///
/// actix-web doesn't allow pretty-print for json so we need to do this manually.
fn create_apub_response<T>(data: &T) -> FastJobResult<HttpResponse>
where
  T: Serialize,
{
  let json = serde_json::to_string_pretty(&WithContext::new(data, FEDERATION_CONTEXT.clone()))?;

  Ok(
    HttpResponse::Ok()
      .content_type(FEDERATION_CONTENT_TYPE)
      .insert_header((VARY, "Accept"))
      .body(json),
  )
}

fn create_apub_tombstone_response<T: Into<Url>>(id: T) -> FastJobResult<HttpResponse> {
  let tombstone = Tombstone::new(id.into());
  let json = serde_json::to_string_pretty(&WithContext::new(
    tombstone,
    FEDERATION_CONTEXT.deref().clone(),
  ))?;

  Ok(
    HttpResponse::Gone()
      .content_type(FEDERATION_CONTENT_TYPE)
      .status(actix_web::http::StatusCode::GONE)
      .insert_header((VARY, "Accept"))
      .body(json),
  )
}

fn redirect_remote_object(url: &DbUrl) -> HttpResponse {
  let mut res = HttpResponse::PermanentRedirect();
  res.insert_header((actix_web::http::header::LOCATION, url.as_str()));
  res.finish()
}

#[derive(Deserialize)]
pub struct ActivityQuery {
  type_: String,
  id: String,
}

/// Return the ActivityPub json representation of a local activity over HTTP.
pub(crate) async fn get_activity(
  info: web::Path<ActivityQuery>,
  context: web::Data<FastJobContext>,
) -> FastJobResult<HttpResponse> {
  let settings = context.settings();
  let activity_id = Url::parse(&format!(
    "{}/activities/{}/{}",
    settings.get_protocol_and_hostname(),
    info.type_,
    info.id
  ))?
  .into();
  let activity = SentActivity::read_from_apub_id(&mut context.pool(), &activity_id).await?;

  let sensitive = activity.sensitive;
  if sensitive {
    Ok(HttpResponse::Forbidden().finish())
  } else {
    create_apub_response(&activity.data)
  }
}

/// Ensure that the community is public and not removed/deleted.
fn check_community_fetchable(community: &Community) -> FastJobResult<()> {
  check_community_removed_or_deleted(community)?;
  if !community.visibility.can_federate() {
    return Err(FastJobErrorType::NotFound.into());
  }
  Ok(())
}

/// Check if posts or comments in the community are allowed to be fetched
async fn check_community_content_fetchable(
  community: &Community,
  request: &HttpRequest,
  context: &Data<FastJobContext>,
) -> FastJobResult<()> {
  use CommunityVisibility::*;
  check_community_removed_or_deleted(community)?;
  match community.visibility {
    Public | Unlisted => Ok(()),
    Private => {
      let signing_actor =
        signing_actor::<SiteOrMultiOrCommunityOrUser>(request, None, context).await?;
      if community.local {
        Ok(
          CommunityFollowerView::check_has_followers_from_instance(
            community.id,
            get_instance_id(&signing_actor),
            &mut context.pool(),
          )
          .await?,
        )
      } else if let Some(followers_url) = community.followers_url.clone() {
        let mut followers_url = followers_url.inner().clone();
        followers_url
          .query_pairs_mut()
          .append_pair("is_follower", signing_actor.id().as_str());
        let req = context.client().get(followers_url.as_str());
        let req = context.sign_request(req, Bytes::new()).await?;
        context.client().execute(req).await?.error_for_status()?;
        Ok(())
      } else {
        Err(FastJobErrorType::NotFound.into())
      }
    }
    LocalOnlyPublic | LocalOnlyPrivate => Err(FastJobErrorType::NotFound.into()),
  }
}

fn check_community_removed_or_deleted(community: &Community) -> FastJobResult<()> {
  if community.deleted || community.removed {
    Err(FastJobErrorType::Deleted)?
  }
  Ok(())
}
