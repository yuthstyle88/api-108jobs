use actix_web::web::{Data, Json};
use lemmy_api_utils::{
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_db_schema::{source::community::CommunityActions, traits::Followable};
use lemmy_db_views_community::api::ApproveCommunityPendingFollower;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::FastJobResult;

pub async fn post_pending_follows_approve(
  data: Json<ApproveCommunityPendingFollower>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let activity_data = if data.approve {
    CommunityActions::approve_follower(
      &mut context.pool(),
      data.community_id,
      data.follower_id,
      local_user_view.person.id,
    )
    .await?;
    SendActivityData::AcceptFollower(data.community_id, data.follower_id)
  } else {
    CommunityActions::unfollow(&mut context.pool(), data.follower_id, data.community_id).await?;
    SendActivityData::RejectFollower(data.community_id, data.follower_id)
  };
  ActivityChannel::submit_activity(activity_data, &context)?;

  Ok(Json(SuccessResponse::default()))
}
