use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_views_community::{
  api::{ListMultiCommunities, ListMultiCommunitiesResponse},
  MultiCommunityView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub async fn list_multi_communities(
  data: Query<ListMultiCommunities>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<ListMultiCommunitiesResponse>> {
  let followed_by = if let Some(true) = data.followed_only {
    local_user_view.map(|l| l.person.id)
  } else {
    None
  };
  let multi_communities =
    MultiCommunityView::list(&mut context.pool(), data.creator_id, followed_by).await?;
  Ok(Json(ListMultiCommunitiesResponse { multi_communities }))
}
