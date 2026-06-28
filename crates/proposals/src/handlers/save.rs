use actix_web::web::{Data, Json};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_core::error::FastJobResult;
use app_108jobs_db::{
  source::proposal::{ProposalActions, ProposalSavedForm},
  traits::Saveable,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_proposal::{
  api::{ProposalResponse, SaveComment},
  ProposalView,
};

pub async fn save_comment(
  data: Json<SaveComment>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ProposalResponse>> {
  let proposal_saved_form = ProposalSavedForm::new(local_user_view.person.id, data.proposal_id);

  if data.save {
    ProposalActions::save(&mut context.pool(), &proposal_saved_form).await?;
  } else {
    ProposalActions::unsave(&mut context.pool(), &proposal_saved_form).await?;
  }

  let comment_id = data.proposal_id;
  let local_instance_id = local_user_view.person.instance_id;
  let proposal_view = ProposalView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  Ok(Json(ProposalResponse { proposal_view }))
}
