use actix_web::web::{Data, Json};
use app_108jobs_api_utils::{build_response::build_proposal_response, context::FastJobContext};
use app_108jobs_core::error::FastJobResult;
use app_108jobs_db::{
  source::{person::PersonActions, proposal::ProposalActions},
  traits::Likeable,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_proposal::{
  api::{CreateCommentLikeRequest, ProposalResponse},
  ProposalView,
};
use std::ops::Deref;

pub async fn like_comment(
  data: Json<CreateCommentLikeRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ProposalResponse>> {
  let local_instance_id = local_user_view.person.instance_id;
  let comment_id = data.proposal_id;
  let my_person_id = local_user_view.person.id;

  let orig_comment = ProposalView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;
  let previous_score = orig_comment.proposal_actions.and_then(|p| p.like_score);

  // Remove any likes first
  ProposalActions::remove_like(&mut context.pool(), my_person_id, comment_id).await?;
  if let Some(previous_score) = previous_score {
    PersonActions::remove_like(
      &mut context.pool(),
      my_person_id,
      orig_comment.creator.id,
      previous_score,
    )
    .await
    // Ignore errors, since a previous_like of zero throws an error
    .ok();
  }

  Ok(Json(
    build_proposal_response(
      context.deref(),
      comment_id,
      Some(local_user_view),
      local_instance_id,
    )
    .await?,
  ))
}
