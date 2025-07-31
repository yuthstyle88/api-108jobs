use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::FastJobContext, utils::is_admin};
use lemmy_db_schema::source::proposal::{Proposal, ProposalUpdateForm};
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_proposal::api::{EditProposal, ProposalResponse};
use lemmy_utils::error::{FastJobError, FastJobErrorType, FastJobResult};

pub async fn update_proposal(
  data: Json<EditProposal>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ProposalResponse>> {
  let proposal_id_to_update = data.id; // Get the ID of the proposal to update
  let existing_proposal = Proposal::read(
    &mut context.pool(), // Pass the pool connection to Crud::read
    proposal_id_to_update,
  )
  .await?;

  if local_user_view.local_user.id != existing_proposal.user_id {
    return Err(FastJobError::from(FastJobErrorType::ValidationError(
      "You must be the proposl owner.".parse()?,
    )));
  }

  // 2. --- Input Validation (using data from EditProposal) ---
  // Perform checks on optional fields if they are provided in the request.
  if let Some(budget_val) = data.budget {
    if budget_val < 0.0 {
      return Err(FastJobError::from(FastJobErrorType::ValidationError(
        "Budget cannot be negative.".parse()?,
      )));
    }
  }
  if let Some(working_days_val) = data.working_days {
    if working_days_val <= 0 {
      return Err(FastJobError::from(FastJobErrorType::ValidationError(
        "Working days must be positive.".parse()?,
      )));
    }
  }

  let update_data_from_request = data.into_inner();

  let update_form = ProposalUpdateForm {
    description: update_data_from_request.description,
    budget: update_data_from_request.budget,
    working_days: update_data_from_request.working_days,
    brief_url: update_data_from_request.brief_url,
    ..Default::default()
  };

  let updated_record = Proposal::update(
    &mut context.pool(), // Pass a new connection from the pool
    proposal_id_to_update,
    &update_form, // Pass the EditProposal as the form
  )
  .await?; // Await the update operation and propagate errors

  // Return a JSON response containing the updated proposal view wrapped in `ProposalResponse`.
  Ok(Json(ProposalResponse {
    proposal: updated_record,
  }))
}
