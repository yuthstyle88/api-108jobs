use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::list_top_up_requests_inner;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_wallet::api::{ListTopUpRequestQuery, ListTopUpRequestResponse};
use lemmy_utils::error::FastJobResult;

pub async fn list_top_up_requests(
  query: Query<ListTopUpRequestQuery>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListTopUpRequestResponse>> {
  let res = list_top_up_requests_inner(
    &mut context.pool(),
    Some(local_user_view.local_user.id),
    query.into_inner(),
  )
  .await?;

  Ok(Json(res))
}
