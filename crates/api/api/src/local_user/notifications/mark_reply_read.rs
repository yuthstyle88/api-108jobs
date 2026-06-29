use actix_web::web::{Data, Json};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_core::error::{FastJobErrorType, FastJobResult};
use app_108jobs_db::{
  source::proposal_reply::{ProposalReply, ProposalReplyUpdateForm},
  traits::Crud,
};
use app_108jobs_db_views_inbox_combined::api::MarkProposalReplyAsRead;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::SuccessResponse;

pub async fn mark_reply_as_read(
  data: Json<MarkProposalReplyAsRead>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let proposal_reply_id = data.proposal_reply_id;
  let read_proposal_reply = ProposalReply::read(&mut context.pool(), proposal_reply_id).await?;

  if local_user_view.person.id != read_proposal_reply.recipient_id {
    Err(FastJobErrorType::CouldntUpdateProposal)?
  }

  let proposal_reply_id = read_proposal_reply.id;
  let read = Some(data.read);

  ProposalReply::update(
    &mut context.pool(),
    proposal_reply_id,
    &ProposalReplyUpdateForm { read },
  )
  .await?;

  Ok(Json(SuccessResponse::default()))
}
