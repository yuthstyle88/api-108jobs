use actix_web::web::{Data, Json, Path};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::newtypes::PostId;
use app_108jobs_db_schema::source::delivery_details::DeliveryDetails;
use app_108jobs_db_schema_file::enums::DeliveryStatus;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_rider::api::{
  DeliveryStatusEvent, DeliveryStatusResponse, UpdateDeliveryStatusRequest,
};
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};

/// PUT /api/v4/deliveries/{postId}/status
///
/// Updates the status of a delivery. The authenticated user must be either:
/// - The assigned rider for this delivery, or
/// - An employer
///
/// Valid status transitions:
/// - Pending → Assigned
/// - Assigned → EnRouteToPickup
/// - EnRouteToPickup → PickedUp
/// - PickedUp → EnRouteToDropoff
/// - EnRouteToDropoff → Delivered
/// - (Any active state) → Cancelled
pub async fn update_delivery_status(
  path: Path<PostId>,
  data: Json<UpdateDeliveryStatusRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<DeliveryStatusResponse>> {
  let post_id = path.into_inner();
  let person_id = local_user_view.person.id;
  let new_status = data.status;

  // Require reason for cancellation
  if new_status == DeliveryStatus::Cancelled
    && data.reason.as_ref().map_or(true, |r| r.trim().is_empty())
  {
    return Err(FastJobErrorType::ReasonIsRequiredWhenCancelling.into());
  }

  // Verify authorization: user must be admin OR the assigned rider
  let is_admin = local_user_view.local_user.admin;
  let is_rider = if !is_admin {
    // Check if this user is the assigned rider for this delivery
    let mut pool = context.pool();
    let result = DeliveryDetails::validate_rider_identity(&mut pool, person_id, post_id).await;
    result.is_ok()
  } else {
    false
  };

  if !is_admin && !is_rider {
    return Err(FastJobErrorType::NotAnActiveRider.into());
  }

  // Get current delivery to check if status is actually changing
  let current_delivery = {
    let mut pool = context.pool();
    DeliveryDetails::get_by_post_id(&mut pool, post_id).await?
  };

  // Check if status is actually changing
  if current_delivery.status == new_status {
    // Idempotent - return current state without error
    return Ok(Json(DeliveryStatusResponse {
      post_id,
      status: new_status,
      cancellation_reason: current_delivery.cancellation_reason,
      updated_at: current_delivery.updated_at,
    }));
  }

  // Update the delivery status
  let updated_delivery = {
    let mut pool = context.pool();
    DeliveryDetails::update_status(&mut pool, post_id, new_status, data.reason.clone()).await?
  };

  let response = DeliveryStatusResponse {
    post_id,
    status: new_status,
    cancellation_reason: updated_delivery.cancellation_reason,
    updated_at: updated_delivery.updated_at,
  };

  // Publish status change event to Redis for WebSocket listeners
  let event = DeliveryStatusEvent {
    kind: "delivery_status_update",
    post_id,
    status: new_status,
    updated_at: updated_delivery.updated_at,
    reason: data.reason.clone(),
  };

  if let Ok(json) = serde_json::to_string(&event) {
    let channel = format!("delivery:status:{}", post_id);
    let mut redis = context.redis().clone();
    if let Err(e) = redis.publish(&channel, &json).await {
      tracing::warn!(
          ?e,
          post_id = %post_id,
          "Failed to publish delivery status update to Redis"
      );
    }
  }

  Ok(Json(response))
}
