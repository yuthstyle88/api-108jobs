use actix_web::web::{Data, Json};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_api_utils::utils::verify_post_creator;
use app_108jobs_db_schema::newtypes::RiderId;
use app_108jobs_db_schema::source::delivery_details::DeliveryDetails;
use app_108jobs_db_schema::source::delivery_rider_rating::DeliveryRiderRating;
use app_108jobs_db_schema::source::post::Post;
use app_108jobs_db_schema::source::rider::Rider;
use app_108jobs_db_schema::traits::Crud;
use app_108jobs_db_schema_file::enums::{DeliveryStatus, PostKind};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_rider::api::{DeliveryRiderRatingData, RateRiderRequest, RateRiderResponse};
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};

/// POST /api/v4/rider/rate
///
/// Rate a rider after completing a delivery.
/// Only the post creator (employer) can rate the assigned rider.
/// The delivery must be in Delivered status to be rated.
pub async fn rate_rider(
  context: Data<FastJobContext>,
  form: Json<RateRiderRequest>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<RateRiderResponse>> {
  let post_id = form.post_id;
  let rider_id = form.rider_id;
  let rating = form.rating;
  let comment = form.comment.clone();
  let employer_person_id = local_user_view.person.id;

  // Validate rating is between 1 and 5
  if !(1..=5).contains(&rating) {
    return Err(FastJobErrorType::RatingMustBeBetween1And5.into());
  }

  // Verify the user is the post creator (employer)
  verify_post_creator(&mut context.pool(), post_id, employer_person_id).await?;

  // Verify the post is a Delivery post
  let post = Post::read(&mut context.pool(), post_id).await?;
  if post.post_kind != PostKind::Delivery {
    return Err(FastJobErrorType::PostIsNotADeliveryPost.into());
  }

  // Verify the delivery is completed
  let delivery = DeliveryDetails::get_by_post_id(&mut context.pool(), post_id).await?;
  if delivery.status != DeliveryStatus::Delivered {
    return Err(FastJobErrorType::DeliveryMustBeCompleted.into());
  }

  // Verify the rider was assigned to this delivery
  if delivery.assigned_rider_id != Some(rider_id) {
    return Err(FastJobErrorType::RiderWasNotAssignedToThisDelivery.into());
  }

  // Create or update the rating
  let rating_record = DeliveryRiderRating::upsert_for_delivery(
    &mut context.pool(),
    post_id,
    employer_person_id,
    rider_id,
    rating,
    comment,
  )
  .await?;

  // Update the rider's aggregated rating
  update_rider_aggregated_rating(&mut context.pool(), rider_id).await?;

  Ok(Json(RateRiderResponse {
    rating: DeliveryRiderRatingData {
      id: rating_record.id.0,
      post_id,
      employer_id: employer_person_id,
      rider_id,
      rating: rating_record.rating,
      comment: rating_record.comment,
      created_at: rating_record.created_at,
      updated_at: rating_record.updated_at,
    },
  }))
}

/// Update the rider's aggregated rating in the rider table
async fn update_rider_aggregated_rating(
  pool: &mut app_108jobs_db_schema::utils::DbPool<'_>,
  rider_id: RiderId,
) -> FastJobResult<()> {
  use app_108jobs_db_schema::source::rider::RiderUpdateForm;

  // Get the average rating
  let average_rating = DeliveryRiderRating::get_average_rating_for_rider(pool, rider_id).await?;

  // Get the total number of ratings
  let ratings = DeliveryRiderRating::get_by_rider_id(pool, rider_id).await?;
  let total_ratings = ratings.len() as i32;

  // Update the rider record with new rating and completed jobs count
  let update_form = RiderUpdateForm {
    rating: average_rating,
    completed_jobs: Some(total_ratings),
    ..Default::default()
  };

  Rider::update(pool, rider_id, &update_form).await?;

  Ok(())
}

/// GET /api/v4/riders/{riderId}/ratings
///
/// Get all ratings for a specific rider.
pub async fn get_rider_ratings(
  rider_id: actix_web::web::Path<RiderId>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<app_108jobs_db_views_rider::api::GetRiderRatingsResponse>> {
  let ratings = DeliveryRiderRating::get_by_rider_id(&mut context.pool(), *rider_id).await?;

  let average_rating = DeliveryRiderRating::get_average_rating_for_rider(&mut context.pool(), *rider_id).await?;
  let total_ratings = ratings.len() as i32;

  let rating_data: Vec<DeliveryRiderRatingData> = ratings
    .into_iter()
    .map(|r| DeliveryRiderRatingData {
      id: r.id.0,
      post_id: r.post_id,
      employer_id: r.employer_id,
      rider_id: r.rider_id,
      rating: r.rating,
      comment: r.comment,
      created_at: r.created_at,
      updated_at: r.updated_at,
    })
    .collect();

  Ok(Json(app_108jobs_db_views_rider::api::GetRiderRatingsResponse {
    ratings: rating_data,
    average_rating,
    total_ratings,
  }))
}
