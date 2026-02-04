use actix_web::web::{Data, Json, Path};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_api_utils::utils::{
  get_active_rider_by_person, verify_comment_author, verify_comment_on_post, verify_post_creator,
};
use app_108jobs_db_schema::newtypes::PostId;
use app_108jobs_db_schema::source::delivery_details::DeliveryDetails;
use app_108jobs_db_schema::utils::get_conn;
use app_108jobs_db_schema_file::enums::DeliveryStatus;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_rider::api::{AssignDeliveryRequest, DeliveryAssignmentEvent};
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_utils::error::FastJobResult;
use chrono::Utc;
use diesel_async::scoped_futures::ScopedFutureExt;

/// POST /api/v4/deliveries/{postId}/assign
///
/// Assign a rider to a delivery from a comment/proposal.
/// Only the post creator (employer) can assign a rider.
///
/// The comment must:
/// - Be on this delivery post
/// - Be authored by the rider being assigned
///
/// The delivery must be in Pending status to be assigned.
///
/// Requires sender and receiver contact information to be provided.
///
/// All database operations are performed in a single transaction to ensure atomicity.
pub async fn assign_delivery_from_proposal(
    path: Path<PostId>,
    context: Data<FastJobContext>,
    form: Json<AssignDeliveryRequest>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
    let post_id = path.into_inner();
    let comment_id = form.comment_id;
    let rider_person_id = form.person_id;
    let sender_name = form.sender_name.clone();
    let sender_phone = form.sender_phone.clone();
    let receiver_name = form.receiver_name.clone();
    let receiver_phone = form.receiver_phone.clone();
    let employer_person_id = local_user_view.person.id;
    let employer_local_user_id = local_user_view.local_user.id;

    // Get connection and run all database operations in a transaction
    let mut pool = context.pool();
    let conn = &mut get_conn(&mut pool).await?;

    let (delivery, rider_id) = conn
        .run_transaction(|conn| {
            async move {
                // Convert connection to DbPool for use with Crud functions
                let mut pool: app_108jobs_db_schema::utils::DbPool<'_> = conn.into();

                // Verify the user is the post creator (employer)
                verify_post_creator(&mut pool, post_id, employer_person_id).await?;

                // Get the comment and verify it's on this post
                let comment = verify_comment_on_post(&mut pool, comment_id, post_id).await?;

                // Verify the comment author matches the provided person_id
                verify_comment_author(&comment, rider_person_id)?;

                // Get the rider from the person_id
                let rider = get_active_rider_by_person(&mut pool, rider_person_id).await?;
                let rider_id = rider.id;

                // Perform the assignment with escrow hold and sender/receiver information
                let delivery = DeliveryDetails::assign_from_comment_with_escrow(
                    &mut pool,
                    post_id,
                    rider_id,
                    employer_local_user_id,
                    employer_person_id,
                    comment_id,
                    sender_name,
                    sender_phone,
                    receiver_name,
                    receiver_phone,
                )
                .await?;

                // Mark the comment proposal as processed (no longer pending)
                // This is done within the same transaction to ensure atomicity
                comment.set_not_pending(&mut pool).await?;

                Ok::<_, app_108jobs_utils::error::FastJobError>((delivery, rider_id))
            }
            .scope_boxed()
        })
        .await?;

    // Publish event to Redis for WebSocket clients (outside transaction)
    let event = DeliveryAssignmentEvent {
        kind: "delivery_assigned",
        post_id,
        rider_id,
        assigned_at: delivery.assigned_at.unwrap_or_else(Utc::now),
        status: DeliveryStatus::Assigned,
    };

    if let Ok(json) = serde_json::to_string(&event) {
        let channel = format!("delivery:{}", post_id);
        let mut redis = context.redis().clone();
        if let Err(e) = redis.publish(&channel, &json).await {
            tracing::warn!(
                ?e,
                post_id = %post_id,
                rider_id = %rider_id,
                "Failed to publish delivery assignment event to Redis"
            );
        }
    }

    Ok(Json(SuccessResponse { success: true }))
}
