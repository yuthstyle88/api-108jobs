use actix_web::web::{Data, Json};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::proposal::Proposal;
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_proposal::api::DeleteProposal;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::FastJobResult;

pub async fn delete_proposal(
  data: Json<DeleteProposal>,
  context: Data<FastJobContext>,
  _local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  Proposal::delete(&mut context.pool(), data.id).await?;

  Ok(Json(SuccessResponse::default()))
}
