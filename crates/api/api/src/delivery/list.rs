use actix_web::web::{Data, Json};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::source::delivery_details::DeliveryDetails;
use app_108jobs_utils::error::FastJobResult;

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
