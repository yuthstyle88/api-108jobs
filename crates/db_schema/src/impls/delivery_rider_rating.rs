use crate::{
  newtypes::{DeliveryRiderRatingId, PersonId, PostId, RiderId},
  source::delivery_rider_rating::{
    DeliveryRiderRating,
    DeliveryRiderRatingInsertForm,
    DeliveryRiderRatingUpdateForm,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};

use diesel::dsl::insert_into;
use diesel::OptionalExtension;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use chrono::Utc;

use app_108jobs_db_schema_file::schema::delivery_rider_rating;
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Crud for DeliveryRiderRating {
  type InsertForm = DeliveryRiderRatingInsertForm;
  type UpdateForm = DeliveryRiderRatingUpdateForm;
  type IdType = DeliveryRiderRatingId;

  async fn create(
    pool: &mut DbPool<'_>,
    form: &Self::InsertForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(delivery_rider_rating::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateDeliveryDetails)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    rating_id: DeliveryRiderRatingId,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    diesel::update(delivery_rider_rating::table.find(rating_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateDeliveryDetails)
  }
}

impl DeliveryRiderRating {
  /// Create or update a rating for a rider by an employer for a specific delivery.
  /// Uses upsert logic based on the unique constraint (post_id, employer_id, rider_id).
  pub async fn upsert_for_delivery(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    employer_id: PersonId,
    rider_id: RiderId,
    rating: i16,
    comment: Option<String>,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    // Check if rating already exists
    let existing = delivery_rider_rating::table
      .filter(delivery_rider_rating::post_id.eq(post_id))
      .filter(delivery_rider_rating::employer_id.eq(employer_id))
      .filter(delivery_rider_rating::rider_id.eq(rider_id))
      .first::<Self>(conn)
      .await
      .optional()
      .map_err(|_| FastJobErrorType::DatabaseError)?;

    match existing {
      Some(existing_rating) => {
        // Update existing rating
        let form = DeliveryRiderRatingUpdateForm {
          rating: Some(rating),
          comment: Some(comment),
          updated_at: Some(Some(Utc::now())),
        };
        Self::update(pool, existing_rating.id, &form).await
      }
      None => {
        // Create new rating
        let form = DeliveryRiderRatingInsertForm {
          post_id,
          employer_id,
          rider_id,
          rating,
          comment,
          created_at: Some(Utc::now()),
          updated_at: None,
        };
        Self::create(pool, &form).await
      }
    }
  }

  /// Get all ratings for a specific rider.
  pub async fn get_by_rider_id(
    pool: &mut DbPool<'_>,
    rider_id: RiderId,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    delivery_rider_rating::table
      .filter(delivery_rider_rating::rider_id.eq(rider_id))
      .order(delivery_rider_rating::created_at.desc())
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  /// Get rating for a specific delivery by employer.
  pub async fn get_for_delivery_by_employer(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    employer_id: PersonId,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;

    delivery_rider_rating::table
      .filter(delivery_rider_rating::post_id.eq(post_id))
      .filter(delivery_rider_rating::employer_id.eq(employer_id))
      .first::<Self>(conn)
      .await
      .optional()
      .map_err(|_| FastJobErrorType::DatabaseError.into())
  }

  /// Get the average rating for a rider.
  pub async fn get_average_rating_for_rider(
    pool: &mut DbPool<'_>,
    rider_id: RiderId,
  ) -> FastJobResult<Option<f64>> {
    let ratings = Self::get_by_rider_id(pool, rider_id).await?;

    if ratings.is_empty() {
      Ok(None)
    } else {
      let sum: i64 = ratings.iter().map(|r| r.rating as i64).sum();
      Ok(Some(sum as f64 / ratings.len() as f64))
    }
  }
}
