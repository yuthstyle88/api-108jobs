use actix_web::web::{Data, Json};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_views_local_user::LocalUserView;

use app_108jobs_db_schema::source::rider::{Rider, RiderInsertForm};
use app_108jobs_db_schema::traits::Crud;
use app_108jobs_db_schema_file::enums::RiderVerificationStatus;
use app_108jobs_db_views_rider::api::{CreateRider, CreateRiderRequest};
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};
use chrono::Utc;

pub async fn create_rider(
  data: Json<CreateRiderRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let data: CreateRider = data.into_inner().try_into()?;

  if Rider::exists_for_user(&mut context.pool(), local_user_view.local_user.id).await? {
    return Err(FastJobErrorType::RiderAlreadyExists)?;
  }

  let rider_form = RiderInsertForm {
    vehicle_type: data.vehicle_type,
    vehicle_plate_number: data.vehicle_plate_number,
    license_number: data.license_number,
    license_expiry_date: data.license_expiry_date,

    is_verified: Some(false),
    is_active: Some(true),
    verification_status: Some(RiderVerificationStatus::Pending),

    rating: Some(0.0),
    completed_jobs: Some(0),
    total_jobs: Some(0),
    total_earnings: Some(0.0),
    pending_earnings: Some(0.0),

    is_online: Some(false),
    accepting_jobs: Some(false),

    joined_at: Some(Utc::now()),

    ..RiderInsertForm::new(
      local_user_view.local_user.id,
      local_user_view.person.id,
      data.vehicle_type,
    )
  };

  let _ = Rider::create(&mut context.pool(), &rider_form).await?;

  Ok(Json(SuccessResponse::default()))
}
