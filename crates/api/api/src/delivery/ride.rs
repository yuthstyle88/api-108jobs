use actix_web::web::{Data, Json, Path, Query};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_api_utils::utils::{check_fetch_limit, get_active_rider_by_person};
use app_108jobs_db_schema::newtypes::RideSessionId;
use app_108jobs_db_schema::source::currency::Currency;
use app_108jobs_db_schema::source::pricing_config::PricingConfig;
use app_108jobs_db_schema::source::ride_session::{RideSession, RideSessionInsertForm, RideSessionUpdateForm};
use app_108jobs_db_schema::traits::Crud;
use app_108jobs_db_schema_file::enums::DeliveryStatus;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_rider::api::{
  CancelRideSessionRequest, CancelRideSessionResponse, ConfirmRideRequest, CreateRideSessionRequest, ListAvailableRides,
  ListAvailableRidesResponse, ListMyRideSessions, ListMyRideSessionsResponse, PricingBreakdown,
  PricingConfigSnapshot, PricingConfigSnapshotResponse, RideMeterEvent, RideMeterResponse, RideSessionResponse,
  RideStatusEvent, UpdateRideMeterRequest, UpdateRideStatusRequest, UpdateRideStatusResponse,
};
use app_108jobs_db_views_rider::ride_session_view::{project_ride_session, RideViewer};
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};
use chrono::Utc;

/// POST /api/v4/rides/create
///
/// Creates a new ride session (taxi-style ride with dynamic pricing).
/// Only the employer (post creator) can create a ride session.
///
/// The ride session will use the active pricing config for the default currency,
/// or a specific pricing config if provided.
///
/// If rider_person_id is provided, the ride will be assigned to that rider directly.
/// Validation: One post can only have one ride session, and one rider can only have one active ride.
pub async fn create_ride_session(
  context: Data<FastJobContext>,
  form: Json<CreateRideSessionRequest>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<RideSessionResponse>> {
  let post_id = form.post_id;
  let employer_id = local_user_view.local_user.id;

  // Check if a ride session already exists for this post
  if RideSession::exists_for_post(&mut context.pool(), post_id).await? {
    return Err(FastJobErrorType::RideSessionAlreadyExistsForPost.into());
  }

  // Get the default pricing config if not specified
  let pricing_config = if let Some(config_id) = form.pricing_config_id {
    PricingConfig::read(&mut context.pool(), config_id).await?
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

  // Lookup rider if rider_person_id is provided
  let (rider_id, initial_status, rider_assigned_at) = if let Some(person_id) = form.rider_person_id {
    let rider = get_active_rider_by_person(&mut context.pool(), person_id).await?;

    // Check if rider already has an active ride session
    if RideSession::has_active_session(&mut context.pool(), rider.id).await? {
      return Err(FastJobErrorType::RiderAlreadyHasActiveRide.into());
    }

    (Some(rider.id), DeliveryStatus::Assigned, Some(Utc::now()))
  } else {
    (None, DeliveryStatus::Pending, None)
  };

  // Create the ride session
  let session_form = RideSessionInsertForm {
    post_id,
    rider_id,
    employer_id,
    pricing_config_id: Some(pricing_config.id),
    pickup_address: form.pickup_address.clone(),
    pickup_lat: form.pickup_lat,
    pickup_lng: form.pickup_lng,
    dropoff_address: form.dropoff_address.clone(),
    dropoff_lat: form.dropoff_lat,
    dropoff_lng: form.dropoff_lng,
    pickup_note: form.pickup_note.clone(),
    passenger_name: form.passenger_name.clone(),
    passenger_phone: form.passenger_phone.clone(),
    payment_method: form.payment_method.clone(),
    payment_status: Some("pending".to_string()),
    status: Some(initial_status),
    requested_at: Some(Utc::now()),
    current_price_coin: Some(base_fare), // Start with base fare
  };

  let session = RideSession::create(&mut context.pool(), &session_form).await?;

  // If rider was assigned, update the rider_assigned_at timestamp
  let session = if rider_assigned_at.is_some() {
    let update_form = RideSessionUpdateForm {
      rider_assigned_at: Some(rider_assigned_at),
      ..Default::default()
    };
    RideSession::update(&mut context.pool(), session.id, &update_form).await?
  } else {
    session
  };

  // Publish event for available riders
  let event = RideStatusEvent {
    kind: if rider_id.is_some() { "ride_assigned" } else { "ride_requested" },
    session_id: session.id,
    post_id,
    status: session.status,
    updated_at: session.created_at,
  };

  publish_ride_event(&context, &event, session.id).await;

  Ok(Json(RideSessionResponse {
    id: session.id,
    post_id,
    rider_id: session.rider_id,
    status: session.status,
    current_price_coin: session.current_price_coin,
    payment_method: session.payment_method,
    payment_status: session.payment_status,
    created_at: session.created_at,
  }))
}

/// POST /api/v4/rides/{sessionId}/confirm
///
/// Rider confirms they are taking this ride (RiderConfirmed status).
/// This is called after the employer assigns the rider via create_ride_session.
pub async fn confirm_ride_assignment(
  path: Path<RideSessionId>,
  context: Data<FastJobContext>,
  _form: Json<ConfirmRideRequest>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<RideSessionResponse>> {
  let session_id = path.into_inner();
  let person_id = local_user_view.person.id;

  // Get the rider for this person
  let rider = get_active_rider_by_person(&mut context.pool(), person_id).await?;

  // Get the current session
  let session = RideSession::read(&mut context.pool(), session_id)
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

  let updated = RideSession::update(&mut context.pool(), session_id, &update_form).await?;

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
    id: updated.id,
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
  path: Path<RideSessionId>,
  data: Json<UpdateRideMeterRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<RideMeterResponse>> {
  let session_id = path.into_inner();
  let elapsed_minutes = data.elapsed_minutes;
  let distance_km = data.distance_km;

  // Get the session
  let session = RideSession::read(&mut context.pool(), session_id)
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

  let updated = RideSession::update(&mut context.pool(), session_id, &update_form).await?;

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

/// GET /api/v4/rides/{sessionId}/pricing-config
///
/// Get the pricing config snapshot for a ride session.
/// Used by rider app to calculate meter updates locally.
/// Only the assigned rider or employer who created the session can access this.
pub async fn get_ride_pricing_config(
  path: Path<RideSessionId>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PricingConfigSnapshotResponse>> {
  let session_id = path.into_inner();
  let person_id = local_user_view.person.id;

  // Get the current session
  let session = RideSession::read(&mut context.pool(), session_id).await?;

  // Check authorization: must be the assigned rider or the employer who created this session
  let rider = get_active_rider_by_person(&mut context.pool(), person_id).await.ok();
  let is_rider = rider.map_or(false, |r| session.rider_id == Some(r.id));
  let is_employer = session.employer_id == local_user_view.local_user.id;

  if !is_rider && !is_employer {
    return Err(FastJobErrorType::NotAnActiveRider.into());
  }

  // Get the pricing config
  let pricing_config_id = session.pricing_config_id
    .ok_or(FastJobErrorType::NotFound)?;

  let pricing_config = PricingConfig::read(&mut context.pool(), pricing_config_id).await?;

  // Get the currency info
  let currency = Currency::read(&mut context.pool(), pricing_config.currency_id).await?;

  Ok(Json(PricingConfigSnapshotResponse {
    session_id,
    pricing_config: PricingConfigSnapshot {
      id: pricing_config.id,
      name: pricing_config.name,
      base_fare_coin: pricing_config.base_fare_coin,
      time_charge_per_minute_coin: pricing_config.time_charge_per_minute_coin,
      minimum_charge_minutes: pricing_config.minimum_charge_minutes,
      distance_charge_per_km_coin: pricing_config.distance_charge_per_km_coin,
      currency_code: currency.code,
      currency_symbol: currency.symbol,
    },
  }))
}

/// GET /api/v4/rides/my-sessions
///
/// List ride sessions for the current rider.
/// Riders can see their assigned and completed rides.
pub async fn list_my_ride_sessions(
  query: Query<ListMyRideSessions>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListMyRideSessionsResponse>> {
  let person_id = local_user_view.person.id;
  let is_admin = local_user_view.local_user.admin;
  let limit = check_fetch_limit(query.limit)?;

  // Get the rider for this person
  let rider = get_active_rider_by_person(&mut context.pool(), person_id).await?;

  // Get ride sessions for this rider
  let sessions = RideSession::list_for_rider(
    &mut context.pool(),
    rider.id,
    query.status,
    Some(limit),
  ).await?;

  // Project to views
  let viewer = RideViewer::Rider(rider.id);
  let rides = sessions
    .iter()
    .map(|session| project_ride_session(session, viewer, person_id, is_admin))
    .collect();

  Ok(Json(ListMyRideSessionsResponse { rides }))
}

/// GET /api/v4/rides/available
///
/// List available ride sessions that riders can accept.
/// Only shows Pending rides with no rider assigned.
pub async fn list_available_rides(
  query: Query<ListAvailableRides>,
  context: Data<FastJobContext>,
  _local_user_view: LocalUserView,
) -> FastJobResult<Json<ListAvailableRidesResponse>> {
  let limit = check_fetch_limit(query.limit)?;

  // Get available rides (Pending, no rider assigned)
  let sessions = RideSession::list_available_for_rider(
    &mut context.pool(),
    Some(limit),
  ).await?;

  // Project to public views (limited info for available rides)
  let rides = sessions
    .iter()
    .map(|session| project_ride_session(session, RideViewer::Public, Default::default(), false))
    .filter_map(|view| match view {
      app_108jobs_db_views_rider::ride_session_view::RideSessionView::Public(public) => Some(public),
      _ => None,
    })
    .collect();

  Ok(Json(ListAvailableRidesResponse { rides }))
}

/// POST /api/v4/rides/{sessionId}/cancel
///
/// Cancel a ride session (employer or rider can cancel).
/// Only the assigned rider or employer who created the session can cancel.
/// Reason for cancellation is required.
pub async fn cancel_ride_session(
  path: Path<RideSessionId>,
  form: Json<CancelRideSessionRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CancelRideSessionResponse>> {
  let session_id = path.into_inner();
  let person_id = local_user_view.person.id;

  // Get the current session
  let session = RideSession::read(&mut context.pool(), session_id).await?;

  // Check authorization: must be the assigned rider or the employer who created this session
    let rider = get_active_rider_by_person(&mut context.pool(), person_id).await.ok();
    let is_rider = rider.map_or(false, |r| session.rider_id == Some(r.id));

    let is_employer = session.employer_id == local_user_view.local_user.id;

  if !is_rider && !is_employer {
    return Err(FastJobErrorType::NotAnActiveRider.into());
  }

  // Check if session can be cancelled (not Delivered or Cancelled)
  if matches!(session.status, DeliveryStatus::Delivered | DeliveryStatus::Cancelled) {
    return Err(FastJobErrorType::CannotCancelCompletedRide.into());
  }

  let cancelled_at = Utc::now();

  // Update session status to Cancelled
  let update_form = RideSessionUpdateForm {
    status: Some(DeliveryStatus::Cancelled),
    cancellation_reason: Some(Some(form.reason.clone())),
    updated_at: Some(Some(cancelled_at)),
    ..Default::default()
  };

  RideSession::update(&mut context.pool(), session_id, &update_form).await?;

  // Publish event
  let event = RideStatusEvent {
    kind: "ride_cancelled",
    session_id,
    post_id: session.post_id,
    status: DeliveryStatus::Cancelled,
    updated_at: cancelled_at,
  };

  publish_ride_event(&context, &event, session_id).await;

  Ok(Json(CancelRideSessionResponse {
    session_id,
    status: DeliveryStatus::Cancelled,
    cancellation_reason: form.reason.clone(),
    cancelled_at,
  }))
}

/// PUT /api/v4/rides/{sessionId}/status
///
/// Updates the status of a ride session. The authenticated user must be either:
/// - The assigned rider for this ride, or
/// - The employer who created this ride session
///
/// Valid status transitions:
/// - RiderConfirmed → EnRouteToPickup
/// - EnRouteToPickup → PickedUp
/// - PickedUp → EnRouteToDropoff
/// - EnRouteToDropoff → Delivered
/// - (Any active state) → Cancelled (use cancel_ride_session instead)
pub async fn update_ride_status(
  path: Path<RideSessionId>,
  form: Json<UpdateRideStatusRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<UpdateRideStatusResponse>> {
  let session_id = path.into_inner();
  let person_id = local_user_view.person.id;
  let new_status = form.status;

  // Get the current session
  let session = RideSession::read(&mut context.pool(), session_id).await?;

  // Check authorization: must be the assigned rider or the employer who created this session
  let rider = get_active_rider_by_person(&mut context.pool(), person_id).await.ok();
  let is_rider = rider.map_or(false, |r| session.rider_id == Some(r.id));
  let is_employer = session.employer_id == local_user_view.local_user.id;

  if !is_rider && !is_employer {
    return Err(FastJobErrorType::NotAnActiveRider.into());
  }

  // For cancellation, use the dedicated cancel endpoint
  if new_status == DeliveryStatus::Cancelled {
    return Err(FastJobErrorType::ReasonIsRequiredWhenCancelling.into());
  }

  // Check if status is actually changing
  if session.status == new_status {
    // Idempotent - return current state without error
    return Ok(Json(UpdateRideStatusResponse {
      session_id,
      status: new_status,
      cancellation_reason: None,
      updated_at: session.updated_at.unwrap_or(session.created_at),
    }));
  }

  // Validate status transitions based on current status
  let valid_transition = match (&session.status, &new_status) {
    (DeliveryStatus::RiderConfirmed, DeliveryStatus::EnRouteToPickup) => true,
    (DeliveryStatus::EnRouteToPickup, DeliveryStatus::PickedUp) => true,
    (DeliveryStatus::PickedUp, DeliveryStatus::EnRouteToDropoff) => true,
    (DeliveryStatus::EnRouteToDropoff, DeliveryStatus::Delivered) => true,
    _ => false,
  };

  if !valid_transition {
    return Err(FastJobErrorType::InvalidDeliveryPost.into());
  }

  let now = Utc::now();

  // Build update form with appropriate timestamp based on new status
  let update_form = RideSessionUpdateForm {
    status: Some(new_status),
    arrived_at_pickup_at: if new_status == DeliveryStatus::EnRouteToPickup {
      Some(Some(now))
    } else {
      None
    },
    ride_started_at: if new_status == DeliveryStatus::PickedUp {
      Some(Some(now))
    } else {
      None
    },
    ride_completed_at: if new_status == DeliveryStatus::Delivered {
      Some(Some(now))
    } else {
      None
    },
    updated_at: Some(Some(now)),
    ..Default::default()
  };

  let _updated = RideSession::update(&mut context.pool(), session_id, &update_form).await?;

  // Publish event
  let event = RideStatusEvent {
    kind: "ride_status_update",
    session_id,
    post_id: session.post_id,
    status: new_status,
    updated_at: now,
  };

  publish_ride_event(&context, &event, session_id).await;

  Ok(Json(UpdateRideStatusResponse {
    session_id,
    status: new_status,
    cancellation_reason: None,
    updated_at: now,
  }))
}

/// Helper function to publish ride status events to Redis
async fn publish_ride_event(context: &FastJobContext, event: &RideStatusEvent, session_id: RideSessionId) {
  if let Ok(json) = serde_json::to_string(&event) {
    let channel = format!("ride:status:{}", session_id.0);
    let mut redis = context.redis().clone();
    if let Err(e) = redis.publish(&channel, &json).await {
      tracing::warn!(
        ?e,
        session_id = session_id.0,
        "Failed to publish ride status update to Redis"
      );
    }
  }
}

/// Helper function to publish ride meter events to Redis
async fn publish_meter_event(context: &FastJobContext, event: &RideMeterEvent, session_id: RideSessionId) {
  if let Ok(json) = serde_json::to_string(&event) {
    let channel = format!("ride:meter:{}", session_id.0);
    let mut redis = context.redis().clone();
    if let Err(e) = redis.publish(&channel, &json).await {
      tracing::warn!(
        ?e,
        session_id = session_id.0,
        "Failed to publish ride meter update to Redis"
      );
    }
  }
}
