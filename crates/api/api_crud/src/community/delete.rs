use actix_web::web::Data;
use actix_web::web::{ Json};
use lemmy_api_utils::utils::{check_community_deleted_removed, is_admin};
use lemmy_api_utils::{
  build_response::build_community_response,
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_db_schema::{
  source::community::{Community, CommunityUpdateForm},
  traits::Crud,
};
use lemmy_db_views_community::api::{CommunityResponse, DeleteCommunity};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub async fn delete_community(
  data: Json<DeleteCommunity>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CommunityResponse>> {
  is_admin(&local_user_view)?;

  let community = Community::read(&mut context.pool(), data.community_id).await?;
  check_community_deleted_removed(&community)?;

  // Do the delete
  let community_id = data.community_id;
  let deleted = data.deleted;
  let community = Community::update(
    &mut context.pool(),
    community_id,
    &CommunityUpdateForm {
      deleted: Some(deleted),
      ..Default::default()
    },
  )
  .await?;

  ActivityChannel::submit_activity(
    SendActivityData::DeleteCommunity(local_user_view.person.clone(), community, data.deleted),
    &context,
  )?;

  build_community_response(&context, local_user_view, community_id).await
}
