use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{context::FastJobContext, utils::check_private_instance};
use lemmy_db_schema::source::{actor_language::CommunityLanguage, community::Community};
use lemmy_db_views_community::{
  api::{CommunityResponse, GetRandomCommunity},
  CommunityView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub async fn get_random_community(
  data: Query<GetRandomCommunity>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<CommunityResponse>> {
  let site_view = context.site_config().get().await?.site_view;

  check_private_instance(&local_user_view, &site_view.local_site)?;

  let local_user = local_user_view.as_ref().map(|u| &u.local_user);

  let random_community_id =
    Community::get_random_community_id(&mut context.pool(), &data.type_, data.self_promotion)
      .await?;

  let community_view =
    CommunityView::read(&mut context.pool(), random_community_id, local_user).await?;

  let discussion_languages =
    CommunityLanguage::read(&mut context.pool(), random_community_id).await?;

  Ok(Json(CommunityResponse {
    community_view,
    discussion_languages,
  }))
}
