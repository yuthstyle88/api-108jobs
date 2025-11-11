use actix_web::web::Data;
use diesel::NotFound;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::{traits::ApubActor};
use lemmy_utils::error::FastJobResult;


/// Resolve actor identifier like `!news@example.com` to user or category object.
///
/// In case the requesting user is logged in and the object was not found locally, it is attempted
/// to fetch via webfinger from the original instance.
pub async fn resolve_ap_identifier<ActorType, DbActor>(
  identifier: &str,
  context: &Data<FastJobContext>,
  include_deleted: bool,
) -> FastJobResult<ActorType>
where
  DbActor: ApubActor + Send + 'static, ActorType: std::convert::From<DbActor>
{
    let identifier = identifier.to_string();
    Ok(
      DbActor::read_from_name(&mut context.pool(), &identifier, include_deleted)
        .await?
        .ok_or(NotFound)?
        .into(),
    )
}

