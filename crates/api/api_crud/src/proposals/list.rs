use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::impls::proposal::{ListProposals, ListProposalsResponse};
use lemmy_db_schema::source::proposal::Proposal;
use lemmy_utils::error::FastJobError;

pub async fn list_proposals(
  data: Query<ListProposals>,
  context: Data<FastJobContext>,
) -> Result<Json<ListProposalsResponse>, FastJobError> {
  // Proposal::list now returns the complete ListProposalsResponse struct
  let response_data: ListProposalsResponse = Proposal::list(&mut context.pool(), &data).await?;

  Ok(Json(response_data))
}
