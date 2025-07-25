use actix_web::web::{Data, Json};
use lemmy_api_utils::{
  build_response::build_community_response,
  context::FastJobContext
  ,
  utils::is_admin,
};
use lemmy_api_utils::utils::check_community_deleted_removed;
use lemmy_db_schema::{
  source::community::{Community, CommunityUpdateForm},
  traits::Crud,
};
use lemmy_db_views_community::api::{CommunityResponse, RemoveCommunity};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub async fn remove_community(
  data: Json<RemoveCommunity>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CommunityResponse>> {
  let community = Community::read(&mut context.pool(), data.community_id).await?;
  check_community_deleted_removed(&community)?;

  // Verify its an admin (only an admin can remove a community)
  is_admin(&local_user_view)?;

  // Do the remove
  let community_id = data.community_id;
  let removed = data.removed;
  Community::update(
    &mut context.pool(),
    community_id,
    &CommunityUpdateForm {
      removed: Some(removed),
      ..Default::default()
    },
  )
  .await?;

  build_community_response(&context, local_user_view, community_id).await
}
