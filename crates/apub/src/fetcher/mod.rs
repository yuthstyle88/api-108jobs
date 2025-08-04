use actix_web::web::Data;
use diesel::NotFound;
use either::Either::*;
use itertools::Itertools;
use lemmy_api_utils::context::FastJobContext;
use lemmy_apub_objects::objects::SiteOrMultiOrCommunityOrUser;
use lemmy_db_schema::{newtypes::InstanceId, traits::ApubActor};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobError, FastJobErrorType, FastJobResult};

pub mod search;

/// Resolve actor identifier like `!news@example.com` to user or community object.
///
/// In case the requesting user is logged in and the object was not found locally, it is attempted
/// to fetch via webfinger from the original instance.
pub async fn resolve_ap_identifier<ActorType, DbActor>(
  identifier: &str,
  context: &Data<FastJobContext>,
  local_user_view: &Option<LocalUserView>,
  include_deleted: bool,
) -> FastJobResult<ActorType>
where
  DbActor: ApubActor + Send + 'static, ActorType: std::convert::From<DbActor>
{
  // remote actor

    let identifier = identifier.to_string();
    Ok(
      DbActor::read_from_name(&mut context.pool(), &identifier, include_deleted)
        .await?
        .ok_or(NotFound)?
        .into(),
    )
}

pub(crate) fn get_instance_id(s: &SiteOrMultiOrCommunityOrUser) -> InstanceId {
  match s {
    Left(site) => site.instance_id,
    Right(Left(user)) => user.instance_id,
    Right(Right(community)) => community.instance_id,
  }
}
