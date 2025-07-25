use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::is_admin;
use lemmy_db_views_community_follower::{
  api::{GetCommunityPendingFollowsCount, GetCommunityPendingFollowsCountResponse},
  CommunityFollowerView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub async fn get_pending_follows_count(
  data: Query<GetCommunityPendingFollowsCount>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<GetCommunityPendingFollowsCountResponse>> {
  is_admin(&local_user_view)?;
  
  let count =
    CommunityFollowerView::count_approval_required(&mut context.pool(), data.community_id).await?;
  Ok(Json(GetCommunityPendingFollowsCountResponse { count }))
}
