use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::{context::FastJobContext, utils::is_admin};
use app_108jobs_db_schema::{source::custom_emoji::CustomEmoji, traits::Crud};
use app_108jobs_db_views_custom_emoji::api::DeleteCustomEmoji;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_utils::error::FastJobResult;

pub async fn delete_custom_emoji(
  data: Json<DeleteCustomEmoji>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  CustomEmoji::delete(&mut context.pool(), data.id).await?;

  Ok(Json(SuccessResponse::default()))
}
