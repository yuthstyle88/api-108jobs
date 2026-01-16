use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::source::login_token::LoginToken;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::ListLoginsResponse;
use app_108jobs_utils::error::FastJobResult;

pub async fn list_logins(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListLoginsResponse>> {
  let logins = LoginToken::list(&mut context.pool(), local_user_view.local_user.id).await?;

  Ok(Json(ListLoginsResponse { logins }))
}
