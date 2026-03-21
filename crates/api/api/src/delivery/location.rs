use actix_web::web::{Data, Json, Path};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::newtypes::{PostId, RiderId};
use app_108jobs_db_schema::source::delivery_details::DeliveryDetails;
use app_108jobs_db_schema::source::trip_location_current::{
  TripLocationCurrent, TripLocationCurrentInsertForm, TripLocationCurrentUpdateForm,
};
use app_108jobs_db_schema::source::trip_location_history::{
  TripLocationHistory, TripLocationHistoryInsertForm,
};
use app_108jobs_db_schema::traits::Crud;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Deserialize)]
pub struct LocationUpdate {
  pub lat: f64,
  pub lng: f64,
  pub heading: Option<f64>,
  pub speed_kmh: Option<f64>,
  pub accuracy_m: Option<f64>,
  pub ts: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocationEvent {
  #[serde(rename = "type")]
  pub kind: String,
  pub post_id: PostId,
  pub rider_id: RiderId,
  pub lat: f64,
  pub lng: f64,
  pub heading: Option<f64>,
  pub speed_kmh: Option<f64>,
  pub accuracy_m: Option<f64>,
  pub ts: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetCurrentLocationResponse {
  pub post_id: PostId,
  pub current: Option<TripLocationCurrent>,
}

/// POST /api/deliveries/{postId}/location
pub async fn post_location(
  path: Path<PostId>,
  data: Json<LocationUpdate>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<()>> {
  let post_id = path.into_inner();
  let person_id = local_user_view.person.id;

  // Verify that the authenticated user is the assigned rider for this delivery
  let rider_id = {
    let mut pool = context.pool();
    DeliveryDetails::validate_rider_identity(&mut pool, person_id, post_id).await?
  };

  // Basic coordinate validation
  if !data.lat.is_finite() || !data.lng.is_finite() {
    return Err(FastJobErrorType::InvalidLatitudeOrLongitude.into());
  }

  let timestamp = data.ts.unwrap_or_else(Utc::now);

  let event = LocationEvent {
    kind: "location_update".to_string(),
    post_id,
    rider_id,
    lat: data.lat,
    lng: data.lng,
    heading: data.heading,
    speed_kmh: data.speed_kmh,
    accuracy_m: data.accuracy_m,
    ts: timestamp,
  };

  // Persist location – best-effort (do not fail the HTTP request on DB error)
  if let Err(e) = persist_location(&context, &event).await {
    tracing::warn!(
        ?e,
        post_id = %post_id,
        rider_id = %rider_id,
        "Failed to persist location update"
    );
  }

  // Publish to Redis pub/sub channel → real-time listeners (WebSocket, etc.)
  let channel = format!("trip:loc:{}", post_id);
  if let Ok(json) = serde_json::to_string(&event) {
    let mut redis = context.redis().clone();
    if let Err(e) = redis.publish(&channel, &json).await {
      tracing::warn!(?e, channel = %channel, "Redis publish failed");
    }

    // Cache latest position (with 24h TTL)
    let cache_key = format!("trip:current:{}", post_id);
    if let Err(e) = redis
      .set_value_with_expiry(&cache_key, &event, 24 * 3600)
      .await
    {
      tracing::warn!(?e, key = %cache_key, "Failed to cache current location");
    }
  }

  Ok(Json(())) // → 204 No Content
}

/// GET /api/deliveries/{postId}/location
/// Returns the latest known location for a trip's assigned rider.
/// Publicly readable (no auth required) to mirror WS endpoint accessibility.
pub async fn get_location(
  path: Path<PostId>,
  context: Data<FastJobContext>,
  _local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<GetCurrentLocationResponse>> {
  let post_id = path.into_inner();

  // 1) Try Redis cache first
  let cache_key = format!("trip:current:{}", post_id);
  let mut redis = context.redis().clone();
  if let Ok(maybe_cached) = redis.get_value::<JsonValue>(&cache_key).await {
    if let Some(v) = maybe_cached {
      let lat = v.get("lat").and_then(|x| x.as_f64());
      let lng = v.get("lng").and_then(|x| x.as_f64());
      if let (Some(lat), Some(lng)) = (lat, lng) {
        let rider_id = v
          .get("rider_id")
          .and_then(|x| x.as_i64())
          .map(|i| RiderId(i as i32))
          .unwrap_or(RiderId(0));
        let heading = v.get("heading").and_then(|x| x.as_f64());
        let speed_kmh = v.get("speed_kmh").and_then(|x| x.as_f64());
        let accuracy_m = v.get("accuracy_m").and_then(|x| x.as_f64());
        let updated_at = v
          .get("ts")
          .and_then(|x| x.as_str())
          .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
          .map(|dt| dt.with_timezone(&Utc))
          .unwrap_or_else(Utc::now);

        let current = TripLocationCurrent {
          post_id,
          rider_id,
          lat,
          lng,
          heading,
          speed_kmh,
          accuracy_m,
          updated_at,
        };

        return Ok(Json(GetCurrentLocationResponse { post_id, current: Some(current) }));
      }
    }
  }

  // 2) Fallback to DB: read from current table if exists
  let mut pool = context.pool();
  let current = match TripLocationCurrent::read(&mut pool, post_id).await {
    Ok(row) => Some(row),
    Err(_) => {
      // Try history latest if current missing
      match TripLocationHistory::list_for_post(&mut pool, post_id, 1).await {
        Ok(list) if !list.is_empty() => {
          let h = &list[0];
          Some(TripLocationCurrent {
            post_id: h.post_id,
            rider_id: h.rider_id,
            lat: h.lat,
            lng: h.lng,
            heading: None,
            speed_kmh: None,
            accuracy_m: None,
            updated_at: h.recorded_at,
          })
        }
        _ => None,
      }
    }
  };

  Ok(Json(GetCurrentLocationResponse { post_id, current }))
}

async fn persist_location(context: &FastJobContext, event: &LocationEvent) -> FastJobResult<()> {
  let mut pool = context.pool(); // or context.pool() depending on exact type

  // ── Update / insert current location ────────────────────────────────────────

  let update_form = TripLocationCurrentUpdateForm {
    lat: Some(event.lat),
    lng: Some(event.lng),
    heading: Some(event.heading),
    speed_kmh: Some(event.speed_kmh),
    accuracy_m: Some(event.accuracy_m),
    updated_at: Some(event.ts),
  };

  let update_result = TripLocationCurrent::update(&mut pool, event.post_id, &update_form).await;

  if update_result.is_err() {
    // Probably doesn't exist yet → try insert
    let insert_form = TripLocationCurrentInsertForm::new(
      event.post_id,
      event.rider_id,
      event.lat,
      event.lng,
      // updated_at is set to now() by derive_new if not provided
    );

    TripLocationCurrent::create(&mut pool, &insert_form)
      .await
      .map_err(|e| {
        tracing::error!(?e, "Both insert and update failed for current location");
        e
      })?;
  }

  // ── Append to history (always insert – we keep full trail) ──────────────────

  let history_form =
    TripLocationHistoryInsertForm::new(event.post_id, event.rider_id, event.lat, event.lng);

  TripLocationHistory::create(&mut pool, &history_form)
    .await
    .map_err(|e| {
      tracing::error!(?e, "Failed to append to location history");
      e
    })?;

  Ok(())
}
