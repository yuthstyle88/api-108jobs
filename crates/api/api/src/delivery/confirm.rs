use actix_web::web::{Data, Json, Path};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_api_utils::utils::verify_post_creator;
use app_108jobs_db_schema::newtypes::PostId;
use app_108jobs_db_schema::source::delivery_details::DeliveryDetails;
use app_108jobs_db_schema_file::enums::DeliveryStatus;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_rider::api::DeliveryStatusEvent;
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};
use app_108jobs_db_views_site::api::SuccessResponse;

/// POST /api/v4/deliveries/{postId}/confirm
///
/// Confirm delivery completion and release payment to the rider.
/// Only the employer (post creator) can confirm and release payment.
///
/// The delivery must be in Delivered status to be confirmed.
///
/// This will:
/// 1. Verify the caller is the employer
/// 2. Release the escrowed funds to the rider's wallet
/// 3. Update the employer_confirmed_at timestamp
pub async fn confirm_delivery_completion(
  path: Path<PostId>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let post_id = path.into_inner();
  let employer_person_id = local_user_view.person.id;

  // Verify the user is the post creator (employer)
  verify_post_creator(&mut context.pool(), post_id, employer_person_id).await?;

  // Get the current delivery to check status
  let current_delivery = DeliveryDetails::get_by_post_id(&mut context.pool(), post_id).await?;

  // Can only confirm Delivered status
  if current_delivery.status != DeliveryStatus::Delivered {
    return Err(FastJobErrorType::CannotConfirmNonDeliveredDelivery.into());
  }

  // Get coin_id and platform wallet_id from context helper methods
  let coin_id = context.get_coin_id().await?;
  let platform_wallet_id = context.get_platform_wallet_id().await?;

  // Confirm completion and release payment
  let updated_delivery = DeliveryDetails::confirm_completion_and_release_payment(
    &mut context.pool(),
    post_id,
    employer_person_id,
    coin_id,
    platform_wallet_id,
  )
  .await?;

  // Publish confirmation event to Redis for WebSocket listeners
  let event = DeliveryStatusEvent {
    kind: "delivery_confirmed",
    post_id,
    status: DeliveryStatus::Delivered,
    updated_at: updated_delivery.updated_at,
    reason: Some("Employer confirmed and payment released".to_string()),
  };

  if let Ok(json) = serde_json::to_string(&event) {
    let channel = format!("delivery:status:{}", post_id);
    let mut redis = context.redis().clone();
    if let Err(e) = redis.publish(&channel, &json).await {
      tracing::warn!(
          ?e,
          post_id = %post_id,
          "Failed to publish delivery confirmation event to Redis"
      );
    }
  }

  Ok(Json(SuccessResponse::default()))
}
