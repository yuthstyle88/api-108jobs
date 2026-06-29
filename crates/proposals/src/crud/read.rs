use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::{
  build_response::build_proposal_response,
  context::FastJobContext,
  utils::check_private_instance,
};
use app_108jobs_core::error::FastJobResult;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_proposal::api::{GetComment, ProposalResponse};

pub async fn get_comment(
  data: Query<GetComment>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<ProposalResponse>> {
  let site_view = context.site_config().get().await?.site_view;
  let local_site = site_view.local_site;
  let local_instance_id = site_view.site.instance_id;

  check_private_instance(&local_user_view, &local_site)?;

  Ok(Json(
    build_proposal_response(&context, data.id, local_user_view, local_instance_id).await?,
  ))
}
