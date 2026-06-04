use actix_web::web::{Data, Json};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::source::rider::{Rider, RiderUpdateForm};
use app_108jobs_db_schema::traits::Crud;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_rider::api::{SetAcceptingRequest, SetOnlineRequest, UpdateRiderRequest};
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};
use chrono::Utc;

/// Resolve the active rider profile owned by the authenticated user, or 404.
async fn current_rider(
  context: &FastJobContext,
  local_user_view: &LocalUserView,
) -> FastJobResult<Rider> {
  Rider::get_by_person_id(&mut context.pool(), local_user_view.person.id)
    .await?
    .ok_or(FastJobErrorType::NotFound.into())
}

/// PUT /riders/profile
/// Update the authenticated rider's own editable profile fields.
pub async fn update_rider(
  data: Json<UpdateRiderRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<Rider>> {
  let rider = current_rider(&context, &local_user_view).await?;

  let form = RiderUpdateForm {
    vehicle_type: data.vehicle_type.clone(),
    // `map(Some)` → only overwrite when the client actually sent the field.
    vehicle_plate_number: data.vehicle_plate_number.clone().map(Some),
    license_number: data.license_number.clone().map(Some),
    license_expiry_date: data.license_expiry_date.map(Some),
    ..Default::default()
  };

  let updated = Rider::update(&mut context.pool(), rider.id, &form).await?;
  Ok(Json(updated))
}

/// PATCH /riders/status/online
/// Toggle the rider's online presence (also bumps last-active liveness).
pub async fn set_online(
  data: Json<SetOnlineRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let rider = current_rider(&context, &local_user_view).await?;

  let form = RiderUpdateForm {
    is_online: Some(data.is_online),
    last_active_at: Some(Some(Utc::now())),
    ..Default::default()
  };
  Rider::update(&mut context.pool(), rider.id, &form).await?;
  Ok(Json(SuccessResponse::default()))
}

/// PATCH /riders/status/accepting
/// Toggle whether the rider is currently accepting jobs.
pub async fn set_accepting(
  data: Json<SetAcceptingRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let rider = current_rider(&context, &local_user_view).await?;

  let form = RiderUpdateForm {
    accepting_jobs: Some(data.accepting_jobs),
    last_active_at: Some(Some(Utc::now())),
    ..Default::default()
  };
  Rider::update(&mut context.pool(), rider.id, &form).await?;
  Ok(Json(SuccessResponse::default()))
}

/// POST /riders/heartbeat
/// Liveness ping — refreshes the rider's last-active timestamp.
pub async fn heartbeat(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let rider = current_rider(&context, &local_user_view).await?;

  let form = RiderUpdateForm {
    last_active_at: Some(Some(Utc::now())),
    ..Default::default()
  };
  Rider::update(&mut context.pool(), rider.id, &form).await?;
  Ok(Json(SuccessResponse::default()))
}
