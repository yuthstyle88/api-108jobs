use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{
  context::FastJobContext,
  utils::check_private_instance,
};
use lemmy_db_schema::source::actor_language::CommunityLanguage;
use lemmy_db_views_community::{
  api::{GetCommunity, GetCommunityResponse},
  CommunityView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};

pub async fn get_community(
  data: Query<GetCommunity>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<GetCommunityResponse>> {
  let local_site = context.site_config().get().await?.site_view.local_site;

  if data.name.is_none() && data.id.is_none() {
    Err(FastJobErrorType::NoIdGiven)?
  }

  check_private_instance(&local_user_view, &local_site)?;

  let local_user = local_user_view.as_ref().map(|u| &u.local_user);

  let community_id = data.id.unwrap();


  let community_view = CommunityView::read(
    &mut context.pool(),
    community_id,
    local_user,
  )
  .await?;

  let community_id = community_view.community.id;
  let discussion_languages = CommunityLanguage::read(&mut context.pool(), community_id).await?;

  Ok(Json(GetCommunityResponse {
    community_view,
    site: None,
    discussion_languages,
  }))
}
