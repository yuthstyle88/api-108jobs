use actix_web::web::{Data, Json, Path};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_api_utils::utils::get_active_rider_by_person;
use app_108jobs_db_schema::newtypes::{PricingConfigId, RideSessionId};
use app_108jobs_db_schema::source::currency::Currency;
use app_108jobs_db_schema::source::pricing_config::PricingConfig;
use app_108jobs_db_schema::source::ride_session::{RideSession, RideSessionInsertForm, RideSessionUpdateForm};
use app_108jobs_db_schema::traits::Crud;
use app_108jobs_db_schema_file::enums::DeliveryStatus;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_rider::api::{
  AcceptRideRequest, ConfirmRideRequest, CreateRideSessionRequest, PricingBreakdown,
  RideMeterEvent, RideMeterResponse, RideSessionResponse, RideStatusEvent, UpdateRideMeterRequest,
};
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};
use chrono::Utc;

/// POST /api/v4/rides/create
///
/// Creates a new ride session (taxi-style ride with dynamic pricing).
/// Only the employer (post creator) can create a ride session.
///
/// The ride session will use the active pricing config for the default currency,
/// or a specific pricing config if provided.
pub async fn create_ride_session(
  context: Data<FastJobContext>,
  form: Json<CreateRideSessionRequest>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<RideSessionResponse>> {
  let post_id = form.post_id;
  let employer_id = local_user_view.local_user.id;

  // Get the default pricing config if not specified
  let pricing_config = if let Some(config_id) = form.pricing_config_id {
    PricingConfig::read(&mut context.pool(), PricingConfigId(config_id)).await?
  } else {
    // Get default currency and its active pricing config
    let currency = Currency::get_default(&mut context.pool())
      .await?
      .ok_or(FastJobErrorType::NotFound)?;
    PricingConfig::get_active_for_currency(&mut context.pool(), currency.id)
      .await?
      .ok_or(FastJobErrorType::NotFound)?
  };

  let base_fare = pricing_config.base_fare_coin;

  // Create the ride session
  let session_form = RideSessionInsertForm {
    post_id,
    rider_id: None,  // No rider assigned yet - will be set when rider accepts
    employer_id,
    pricing_config_id: Some(pricing_config.id),
    pickup_address: form.pickup_address.clone(),
    pickup_lat: form.pickup_lat,
    pickup_lng: form.pickup_lng,
    dropoff_address: form.dropoff_address.clone(),
    dropoff_lat: form.dropoff_lat,
    dropoff_lng: form.dropoff_lng,
    pickup_note: form.pickup_note.clone(),
    payment_method: form.payment_method.clone(),
    payment_status: Some("pending".to_string()),
    status: Some(DeliveryStatus::Pending),
    requested_at: Some(Utc::now()),
    current_price_coin: Some(base_fare), // Start with base fare
  };

  let session = RideSession::create(&mut context.pool(), &session_form).await?;

  // Publish event for available riders
  let event = RideStatusEvent {
    kind: "ride_requested",
    session_id: session.id.0,
    post_id,
    status: DeliveryStatus::Pending,
    updated_at: session.created_at,
  };

  publish_ride_event(&context, &event, session.id.0).await;

  Ok(Json(RideSessionResponse {
    id: session.id.0,
    post_id,
    rider_id: session.rider_id,
    status: session.status,
    current_price_coin: session.current_price_coin,
    payment_method: session.payment_method,
    payment_status: session.payment_status,
    created_at: session.created_at,
  }))
}

/// POST /api/v4/rides/{sessionId}/accept
///
/// Rider accepts a ride assignment.
/// Only verified riders can accept rides.
pub async fn accept_ride_assignment(
  path: Path<i32>,
  context: Data<FastJobContext>,
  _form: Json<AcceptRideRequest>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<RideSessionResponse>> {
  let session_id = path.into_inner();
  let person_id = local_user_view.person.id;

  // Get the rider for this person
  let rider = get_active_rider_by_person(&mut context.pool(), person_id).await?;

  // Get the current session
  let session = RideSession::read(&mut context.pool(), RideSessionId(session_id))
    .await?;

  // Check if session is in Pending status
  if session.status != DeliveryStatus::Pending {
    return Err(FastJobErrorType::DeliveryIsNotActive.into());
  }

  // Update session with rider and status
  let update_form = RideSessionUpdateForm {
    rider_id: Some(Some(rider.id)),  // Nullable column, so Some(Some())
    status: Some(DeliveryStatus::Assigned),
    rider_assigned_at: Some(Some(Utc::now())),
    ..Default::default()
  };

  let updated = RideSession::update(
    &mut context.pool(),
    RideSessionId(session_id),
    &update_form,
  )
  .await?;

  // Publish event
  let event = RideStatusEvent {
    kind: "ride_accepted",
    session_id,
    post_id: session.post_id,
    status: DeliveryStatus::Assigned,
    updated_at: updated.updated_at.unwrap_or_else(Utc::now),
  };

  publish_ride_event(&context, &event, session_id).await;

  Ok(Json(RideSessionResponse {
    id: updated.id.0,
    post_id: updated.post_id,
    rider_id: updated.rider_id,
    status: updated.status,
    current_price_coin: updated.current_price_coin,
    payment_method: updated.payment_method,
    payment_status: updated.payment_status,
    created_at: updated.created_at,
  }))
}

/// POST /api/v4/rides/{sessionId}/confirm
///
/// Rider confirms they are taking this ride (RiderConfirmed status).
/// This is specific to taxi-style rides where the rider needs to confirm
/// after the employer assigns them.
pub async fn confirm_ride_assignment(
  path: Path<i32>,
  context: Data<FastJobContext>,
  _form: Json<ConfirmRideRequest>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<RideSessionResponse>> {
  let session_id = path.into_inner();
  let person_id = local_user_view.person.id;

  // Get the rider for this person
  let rider = get_active_rider_by_person(&mut context.pool(), person_id).await?;

  // Get the current session
  let session = RideSession::read(&mut context.pool(), RideSessionId(session_id))
    .await?;

  // Verify this rider is assigned to this session
  if session.rider_id != Some(rider.id) {
    return Err(FastJobErrorType::NotAnActiveRider.into());
  }

  // Check if session is in Assigned status
  if session.status != DeliveryStatus::Assigned {
    return Err(FastJobErrorType::DeliveryIsNotActive.into());
  }

  // Update session status to RiderConfirmed
  let update_form = RideSessionUpdateForm {
    status: Some(DeliveryStatus::RiderConfirmed),
    rider_confirmed_at: Some(Some(Utc::now())),
    ..Default::default()
  };

  let updated = RideSession::update(
    &mut context.pool(),
    RideSessionId(session_id),
    &update_form,
  )
  .await?;

  // Publish event
  let event = RideStatusEvent {
    kind: "ride_confirmed",
    session_id,
    post_id: session.post_id,
    status: DeliveryStatus::RiderConfirmed,
    updated_at: updated.updated_at.unwrap_or_else(Utc::now),
  };

  publish_ride_event(&context, &event, session_id).await;

  Ok(Json(RideSessionResponse {
    id: updated.id.0,
    post_id: updated.post_id,
    rider_id: updated.rider_id,
    status: updated.status,
    current_price_coin: updated.current_price_coin,
    payment_method: updated.payment_method,
    payment_status: updated.payment_status,
    created_at: updated.created_at,
  }))
}

/// PUT /api/v4/rides/{sessionId}/meter
///
/// Updates the ride meter with current elapsed time and distance.
/// Calculates new price based on pricing config.
///
/// Can be called by either the rider (real-time GPS updates) or
/// the system based on time elapsed.
pub async fn update_ride_meter(
  path: Path<i32>,
  data: Json<UpdateRideMeterRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<RideMeterResponse>> {
  let session_id = path.into_inner();
  let elapsed_minutes = data.elapsed_minutes;
  let distance_km = data.distance_km;

  // Get the session
  let session = RideSession::read(&mut context.pool(), RideSessionId(session_id))
    .await?;

  // Verify authorization: must be the assigned rider or admin
  let is_admin = local_user_view.local_user.admin;
  let is_rider = if !is_admin {
    let rider = get_active_rider_by_person(&mut context.pool(), local_user_view.person.id).await.ok();
    rider.map_or(false, |r| session.rider_id == Some(r.id))
  } else {
    false
  };

  if !is_admin && !is_rider {
    return Err(FastJobErrorType::NotAnActiveRider.into());
  }

  // Get pricing config
  let pricing_config_id = session.pricing_config_id.ok_or(FastJobErrorType::NotFound)?;
  let pricing_config = PricingConfig::read(&mut context.pool(), pricing_config_id).await?;

  // Calculate pricing
  let time_charge_coin = ((elapsed_minutes / pricing_config.minimum_charge_minutes).max(1)
    * pricing_config.time_charge_per_minute_coin) as i32;
  let distance_charge_coin = (distance_km * pricing_config.distance_charge_per_km_coin as f64) as i32;
  let total_coin = pricing_config.base_fare_coin + time_charge_coin + distance_charge_coin;

  // Update session
  let update_form = RideSessionUpdateForm {
    current_price_coin: Some(total_coin),
    total_distance_km: Some(Some(distance_km)),
    total_duration_minutes: Some(Some(elapsed_minutes)),
    ..Default::default()
  };

  let updated = RideSession::update(
    &mut context.pool(),
    RideSessionId(session_id),
    &update_form,
  )
  .await?;

  // Get currency for display
  let currency = Currency::read(&mut context.pool(), pricing_config.currency_id).await?;
  let formatted_price = currency.format_coins(total_coin);

  let breakdown = PricingBreakdown {
    base_fare_coin: pricing_config.base_fare_coin,
    time_charge_coin,
    distance_charge_coin,
    total_coin,
    formatted_price,
    currency_code: currency.code,
  };

  // Publish meter update event
  let event = RideMeterEvent {
    kind: "ride_meter_update",
    session_id,
    post_id: session.post_id,
    current_price_coin: total_coin,
    elapsed_minutes,
    distance_km,
    updated_at: updated.updated_at.unwrap_or_else(Utc::now),
  };

  publish_meter_event(&context, &event, session_id).await;

  Ok(Json(RideMeterResponse {
    session_id,
    current_price_coin: total_coin,
    elapsed_minutes,
    distance_km,
    breakdown,
  }))
}

/// Helper function to publish ride status events to Redis
async fn publish_ride_event(context: &FastJobContext, event: &RideStatusEvent, session_id: i32) {
  if let Ok(json) = serde_json::to_string(&event) {
    let channel = format!("ride:status:{}", session_id);
    let mut redis = context.redis().clone();
    if let Err(e) = redis.publish(&channel, &json).await {
      tracing::warn!(
        ?e,
        session_id,
        "Failed to publish ride status update to Redis"
      );
    }
  }
}

/// Helper function to publish ride meter events to Redis
async fn publish_meter_event(context: &FastJobContext, event: &RideMeterEvent, session_id: i32) {
  if let Ok(json) = serde_json::to_string(&event) {
    let channel = format!("ride:meter:{}", session_id);
    let mut redis = context.redis().clone();
    if let Err(e) = redis.publish(&channel, &json).await {
      tracing::warn!(
        ?e,
        session_id,
        "Failed to publish ride meter update to Redis"
      );
    }
  }
}
