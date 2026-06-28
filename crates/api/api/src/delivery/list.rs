use actix_web::web::{Data, Json, Path};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_core::error::FastJobResult;
use app_108jobs_db_schema::{
  newtypes::PostId,
  source::delivery_details::{DeliveryDetails, DeliveryDetailsPrivate},
};
use app_108jobs_db_views_local_user::LocalUserView;

/// GET /deliveries/active
///
/// Returns a list of all active deliveries (not Cancelled or Delivered).
/// Accessible by authenticated users (riders, employers, admins).
pub async fn get_active_deliveries(
  context: Data<FastJobContext>,
) -> FastJobResult<Json<Vec<DeliveryDetails>>> {
  let mut pool = context.pool();
  let active_deliveries = DeliveryDetails::get_all_active(&mut pool).await?;

  Ok(Json(active_deliveries))
}

/// GET /deliveries/completed
///
/// Returns a list of all completed deliveries (status = Delivered).
/// Accessible by authenticated users (riders, employers, admins).
pub async fn get_completed_deliveries(
  context: Data<FastJobContext>,
) -> FastJobResult<Json<Vec<DeliveryDetails>>> {
  let mut pool = context.pool();
  let completed_deliveries = DeliveryDetails::get_all_completed(&mut pool).await?;

  Ok(Json(completed_deliveries))
}

/// GET /deliveries/cancelled
///
/// Returns a list of all cancelled deliveries (status = Cancelled).
/// Accessible by authenticated users (riders, employers, admins).
pub async fn get_cancelled_deliveries(
  context: Data<FastJobContext>,
) -> FastJobResult<Json<Vec<DeliveryDetails>>> {
  let mut pool = context.pool();
  let cancelled_deliveries = DeliveryDetails::get_all_cancelled(&mut pool).await?;

  Ok(Json(cancelled_deliveries))
}

/// GET /deliveries/{postId}
///
/// Returns a single delivery's details by its post id. 404 if no delivery
/// exists for that post. Same `DeliveryDetails` shape as the list endpoints.
/// Accessible by authenticated users (riders, employers, admins).
pub async fn get_delivery(
  path: Path<PostId>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<DeliveryDetails>> {
  let post_id = path.into_inner();
  let mut pool = context.pool();
  let delivery = DeliveryDetails::get_by_post_id(&mut pool, post_id).await?;

  Ok(Json(delivery))
}

/// GET /api/v4/account/deliveries
///
/// Returns all deliveries owned by the authenticated employer (post.creator_id = caller),
/// all statuses, ordered by created_at descending. Returns the full private shape.
pub async fn list_employer_deliveries(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<Vec<DeliveryDetailsPrivate>>> {
  let deliveries =
    DeliveryDetails::list_by_employer(&mut context.pool(), local_user_view.person.id).await?;
  Ok(Json(
    deliveries.into_iter().map(|d| d.to_private()).collect(),
  ))
}

/// GET /api/v4/account/deliveries/{postId}
///
/// Returns a single delivery owned by the authenticated employer.
/// Returns 404 if the delivery does not exist or is not owned by the caller.
pub async fn get_employer_delivery(
  path: Path<PostId>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<DeliveryDetailsPrivate>> {
  let post_id = path.into_inner();
  let delivery = DeliveryDetails::get_by_post_id_for_employer(
    &mut context.pool(),
    post_id,
    local_user_view.person.id,
  )
  .await?;
  Ok(Json(delivery.to_private()))
}
