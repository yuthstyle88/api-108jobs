use crate::{
  newtypes::{PersonId, RiderId},
  source::rider::{Rider, RiderInsertForm, RiderUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};

use diesel::dsl::{exists, select};
use diesel::{
  dsl::{insert_into, update},
  ExpressionMethods, QueryDsl, SelectableHelper, OptionalExtension,
};
use diesel_async::RunQueryDsl;

use crate::newtypes::LocalUserId;
use app_108jobs_db_schema_file::schema::rider;
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Crud for Rider {
  type InsertForm = RiderInsertForm;
  type UpdateForm = RiderUpdateForm;
  type IdType = RiderId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(rider::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateRider)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    rider_id: RiderId,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    update(rider::table.find(rider_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateRider)
  }
}

impl Rider {
  pub async fn read(pool: &mut DbPool<'_>, rider_id: RiderId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    rider::table
      .find(rider_id)
      .select(Self::as_select())
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn get_by_person_id(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;

    let result = rider::table
      .filter(rider::person_id.eq(person_id.0))
      .filter(rider::is_active.eq(true))
      .first::<Self>(conn)
      .await
      .optional()
      .map_err(|_| FastJobErrorType::DatabaseError)?;

    Ok(result)
  }

  pub async fn exists_for_user(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
  ) -> FastJobResult<bool> {
    let conn = &mut get_conn(pool).await?;

    select(exists(
      rider::table.filter(rider::user_id.eq(local_user_id)),
    ))
    .get_result::<bool>(conn)
    .await
    .with_fastjob_type(FastJobErrorType::NotFound)
  }
}
