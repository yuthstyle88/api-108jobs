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
use app_108jobs_db_schema_file::enums::DeliveryStatus;

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

    let result = app_108jobs_db_schema_file::schema::ride_session::table
      .filter(app_108jobs_db_schema_file::schema::ride_session::post_id.eq(post_id))
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

    app_108jobs_db_schema_file::schema::ride_session::table
      .filter(app_108jobs_db_schema_file::schema::ride_session::rider_id.eq(rider_id))
      .order(app_108jobs_db_schema_file::schema::ride_session::created_at.desc())
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  pub async fn get_by_employer(
    pool: &mut DbPool<'_>,
    employer_id: LocalUserId,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    app_108jobs_db_schema_file::schema::ride_session::table
      .filter(app_108jobs_db_schema_file::schema::ride_session::employer_id.eq(employer_id))
      .order(app_108jobs_db_schema_file::schema::ride_session::created_at.desc())
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  pub async fn get_active_by_rider(
    pool: &mut DbPool<'_>,
    rider_id: RiderId,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;

    let result = app_108jobs_db_schema_file::schema::ride_session::table
      .filter(app_108jobs_db_schema_file::schema::ride_session::rider_id.eq(rider_id))
      .filter(app_108jobs_db_schema_file::schema::ride_session::status.eq(DeliveryStatus::PickedUp))
      .first::<Self>(conn)
      .await
      .optional()
      .map_err(|_| FastJobErrorType::DatabaseError)?;

    Ok(result)
  }
}
