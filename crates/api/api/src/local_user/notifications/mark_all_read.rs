use actix_web::web::{Data, Json};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_core::error::FastJobResult;
use app_108jobs_db::source::{
  person_post_mention::PersonPostMention,
  person_proposal_mention::PersonProposalMention,
  proposal_reply::ProposalReply,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::SuccessResponse;

pub async fn mark_all_notifications_read(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let person_id = local_user_view.person.id;

  // Mark all proposal_replies as read
  ProposalReply::mark_all_as_read(&mut context.pool(), person_id).await?;

  // Mark all proposal mentions as read
  PersonProposalMention::mark_all_as_read(&mut context.pool(), person_id).await?;

  // Mark all post mentions as read
  PersonPostMention::mark_all_as_read(&mut context.pool(), person_id).await?;

  Ok(Json(SuccessResponse::default()))
}
