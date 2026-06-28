use actix_web::web::{Data, Json};
use app_108jobs_api_utils::{context::FastJobContext, utils::is_admin};
use app_108jobs_core::error::FastJobError;
use app_108jobs_db_schema::{source::tagline::Tagline, traits::Crud};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::{DeleteTaglineRequest, SuccessResponse};

pub async fn delete_tagline(
  data: Json<DeleteTaglineRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> Result<Json<SuccessResponse>, FastJobError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  Tagline::delete(&mut context.pool(), data.id).await?;

  Ok(Json(SuccessResponse::default()))
}
