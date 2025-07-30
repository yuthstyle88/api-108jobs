use actix_web::web::{Data, Json};
use actix_web::HttpRequest;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::proposal::{Proposal, ProposalInsertForm};
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_proposal::{CreateProposalRequest, CreateProposalResponse};
use lemmy_utils::error::{FastJobError, FastJobErrorType, FastJobResult};
use lemmy_db_schema::source::post::Post;

pub async fn create_proposal(
  data: Json<CreateProposalRequest>,
  _req: HttpRequest,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CreateProposalResponse>> {
  let local_user_view = LocalUserView::read(&mut context.pool(), local_user_view.local_user.id)
    .await
    .map_err(|e| {
      FastJobError::from(FastJobErrorType::DatabaseError)
    })?;

  let post = Post::read_xx(&mut context.pool(), data.post_id).await?;

  // if !post.is_open {
  //   return Err(FastJobError::from(FastJobErrorType::ValidationError(
  //     "The job post must be open to send proposal".to_string(),
  //   )));
  // }

  if data.budget <= 0.0 {
    return Err(FastJobError::from(FastJobErrorType::ValidationError(
      "Budget must be greater than 0".to_string(),
    )));
  }
  if data.working_days <= 0 {
    return Err(FastJobError::from(FastJobErrorType::ValidationError(
      "Working days must be greater than 0".to_string(),
    )));
  }
  if data.description.trim().is_empty() {
    return Err(FastJobError::from(FastJobErrorType::ValidationError(
      "Description cannot be empty".to_string(),
    )));
  }

  let existing_proposal = Proposal::find_by_user_and_job(
    &mut context.pool(),
    local_user_view.local_user.id,
    data.post_id,
  )
  .await?;

  if let Some(proposal) = existing_proposal {
    return Err(FastJobError::from(FastJobErrorType::ValidationError(
      format!(
        "You have already submitted proposal ID {:?} for this job post",
        proposal.id
      ),
    )));
  }
  let proposal_form = ProposalInsertForm {
    description: data.description.clone(),
    budget: data.budget,
    working_days: data.working_days,
    brief_url: data.brief_url.clone(),
    community_id: data.community_id,
    user_id: local_user_view.local_user.id,
    post_id: data.post_id,
  };

    let inserted_proposal = Proposal::create(&mut context.pool(), &proposal_form).await?;

  let response = CreateProposalResponse {
    id: inserted_proposal.id,
    description: inserted_proposal.description,
    budget: inserted_proposal.budget,
    working_days: inserted_proposal.working_days,
    brief_url: inserted_proposal.brief_url,
    community_id: inserted_proposal.community_id,
    user_id: inserted_proposal.user_id,
    post_id: inserted_proposal.post_id,
    created_at: inserted_proposal.created_at,
    updated_at: inserted_proposal.updated_at,
  };

  Ok(Json(response))
}
