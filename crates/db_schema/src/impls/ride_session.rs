use crate::{
  newtypes::{LocalUserId, PostId, RiderId, RideSessionId},
  source::ride_session::{RideSession, RideSessionInsertForm, RideSessionUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};

use diesel::dsl::{insert_into, update};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use app_108jobs_db_schema_file::enums::TripStatus;
use app_108jobs_db_schema_file::schema::ride_session;

impl Crud for RideSession {
  type InsertForm = RideSessionInsertForm;
  type UpdateForm = RideSessionUpdateForm;
  type IdType = RideSessionId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(app_108jobs_db_schema_file::schema::ride_session::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateRideSession)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    session_id: RideSessionId,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    update(app_108jobs_db_schema_file::schema::ride_session::table.find(session_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateRideSession)
  }
}

impl RideSession {
  pub async fn get_by_post(pool: &mut DbPool<'_>, post_id: PostId) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;

    let result = ride_session::table
      .filter(ride_session::post_id.eq(post_id))
      .first::<Self>(conn)
      .await
      .optional()
      .map_err(|_| FastJobErrorType::DatabaseError)?;

    Ok(result)
  }

  pub async fn get_by_rider(
    pool: &mut DbPool<'_>,
    rider_id: RiderId,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    ride_session::table
      .filter(ride_session::rider_id.eq(rider_id))
      .order(ride_session::created_at.desc())
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  pub async fn get_by_employer(
    pool: &mut DbPool<'_>,
    employer_id: LocalUserId,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    ride_session::table
      .filter(ride_session::employer_id.eq(employer_id))
      .order(ride_session::created_at.desc())
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  pub async fn get_active_by_rider(
    pool: &mut DbPool<'_>,
    rider_id: RiderId,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;

    let result = ride_session::table
      .filter(ride_session::rider_id.eq(rider_id))
      .filter(ride_session::status.eq(TripStatus::PickedUp))
      .first::<Self>(conn)
      .await
      .optional()
      .map_err(|_| FastJobErrorType::DatabaseError)?;

    Ok(result)
  }

  /// List ride sessions for a rider with optional status filter and pagination
  pub async fn list_for_rider(
    pool: &mut DbPool<'_>,
    rider_id: RiderId,
    status: Option<TripStatus>,
    limit: Option<i64>,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit.unwrap_or(20);

    let mut query = ride_session::table
      .filter(ride_session::rider_id.eq(rider_id))
      .order(ride_session::created_at.desc())
      .limit(limit)
      .into_boxed();

    if let Some(s) = status {
      query = query.filter(ride_session::status.eq(s));
    }

    query
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  /// List available ride sessions that riders can accept (Pending status, no rider assigned)
  pub async fn list_available_for_rider(
    pool: &mut DbPool<'_>,
    limit: Option<i64>,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit.unwrap_or(20);

    ride_session::table
      .filter(ride_session::status.eq(TripStatus::Pending))
      .filter(ride_session::rider_id.is_null())
      .order(ride_session::created_at.desc())
      .limit(limit)
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  /// Check if a rider has any active (non-terminal) ride sessions
  /// Active statuses: Pending, Assigned, EnRouteToPickup, PickedUp, EnRouteToDropoff, RiderConfirmed
  /// Terminal statuses: Delivered, Cancelled
  pub async fn has_active_session(
    pool: &mut DbPool<'_>,
    rider_id: RiderId,
  ) -> FastJobResult<bool> {
    let conn = &mut get_conn(pool).await?;

    let active_statuses = vec![
      TripStatus::Pending,
      TripStatus::Assigned,
      TripStatus::EnRouteToPickup,
      TripStatus::PickedUp,
      TripStatus::EnRouteToDropoff,
      TripStatus::RiderConfirmed,
    ];

    let count: i64 = ride_session::table
      .filter(ride_session::rider_id.eq(rider_id))
      .filter(ride_session::status.eq_any(active_statuses))
      .count()
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;

    Ok(count > 0)
  }

  /// Check if a post already has a ride session
  pub async fn exists_for_post(
    pool: &mut DbPool<'_>,
    post_id: PostId,
  ) -> FastJobResult<bool> {
    let conn = &mut get_conn(pool).await?;

    let count: i64 = ride_session::table
      .filter(ride_session::post_id.eq(post_id))
      .count()
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;

    Ok(count > 0)
  }
}
