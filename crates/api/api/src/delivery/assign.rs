use actix_web::web::{Data, Json, Path};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::newtypes::PostId;
use app_108jobs_db_schema::source::comment::Comment;
use app_108jobs_db_schema::source::delivery_details::DeliveryDetails;
use app_108jobs_db_schema::source::post::Post;
use app_108jobs_db_schema::source::rider::Rider;
use app_108jobs_db_schema::traits::Crud;
use app_108jobs_db_schema_file::enums::DeliveryStatus;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_rider::api::{AssignDeliveryRequest, DeliveryAssignmentEvent};
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};
use chrono::Utc;

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

    // Verify the user is the post creator (employer)
    let post = {
        let mut pool = context.pool();
        Post::read(&mut pool, post_id).await?
    };

    if post.creator_id != employer_person_id {
        return Err(FastJobErrorType::InvalidField(
            "Only the post creator can assign a rider".to_string(),
        )
        .into());
    }

    // Get the comment to verify it's on this post
    let comment = {
        let mut pool = context.pool();
        Comment::read(&mut pool, comment_id).await?
    };

    // Verify the comment is on this post
    if comment.post_id != post_id {
        return Err(FastJobErrorType::InvalidField(
            "Comment is not on this delivery post".to_string(),
        )
        .into());
    }

    // Verify the comment author matches the provided person_id
    if comment.creator_id != rider_person_id.into() {
        return Err(FastJobErrorType::InvalidField(
            "Comment author must match the provided person_id".to_string(),
        )
        .into());
    }

    // Get the rider from the person_id
    let rider = {
        let mut pool = context.pool();
        Rider::get_by_person_id(&mut pool, rider_person_id).await?
            .ok_or(FastJobErrorType::InvalidField(
                "Person is not an active rider".to_string(),
            ))?
    };
    let rider_id = rider.id;

    // Perform the assignment with sender/receiver information
    let delivery = {
        let mut pool = context.pool();
        DeliveryDetails::assign_from_comment(
            &mut pool,
            post_id,
            rider_id,
            employer_person_id,
            comment_id,
            sender_name,
            sender_phone,
            receiver_name,
            receiver_phone,
        )
        .await?
    };

    // Mark the comment proposal as processed (no longer pending)
    comment.set_not_pending(&mut context.pool()).await?;

    // Publish event to Redis for WebSocket clients
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
