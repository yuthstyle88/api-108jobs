use actix_web::web::{Data, Json};
use app_108jobs_api_utils::{context::FastJobContext, utils::is_admin};
use app_108jobs_db_schema::source::rider::{Rider, RiderUpdateForm};
use app_108jobs_db_schema::traits::Crud;
use app_108jobs_db_schema_file::enums::RiderVerificationStatus;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_rider::api::AdminVerifyRiderRequest;
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_utils::error::FastJobResult;
use chrono::Utc;

pub async fn admin_verify_rider(
  data: Json<AdminVerifyRiderRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  // Ensure caller is admin
  is_admin(&local_user_view)?;

  let AdminVerifyRiderRequest {
    rider_id,
    approve,
    reason: _,
  } = data.into_inner();

  let update_form = if approve {
    RiderUpdateForm {
      is_verified: Some(true),
      verification_status: Some(RiderVerificationStatus::Verified),
      verified_at: Some(Some(Utc::now())),
      ..Default::default()
    }
  } else {
    RiderUpdateForm {
      is_verified: Some(false),
      verification_status: Some(RiderVerificationStatus::Rejected),
      verified_at: Some(None),
      ..Default::default()
    }
  };

  let _ = Rider::update(&mut context.pool(), rider_id, &update_form).await?;

  Ok(Json(SuccessResponse::default()))
}
