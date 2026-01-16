use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::{context::FastJobContext, utils::is_admin};
use app_108jobs_db_schema::{source::tagline::Tagline, traits::Crud};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::{DeleteTagline, SuccessResponse};
use app_108jobs_utils::error::FastJobError;

pub async fn delete_tagline(
  data: Json<DeleteTagline>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> Result<Json<SuccessResponse>, FastJobError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  Tagline::delete(&mut context.pool(), data.id).await?;

  Ok(Json(SuccessResponse::default()))
}
