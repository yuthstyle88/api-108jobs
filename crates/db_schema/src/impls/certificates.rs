use crate::{
  newtypes::PersonId,
  source::certificates::{Certificates, CertificatesInsertForm},
  utils::{get_conn, DbPool},
};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::certificates;
use lemmy_utils::error::{FastJobErrorExt, FastJobResult};

impl Certificates {
  pub async fn create(pool: &mut DbPool<'_>, form: &CertificatesInsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(certificates::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }

  pub async fn read_by_person_id(pool: &mut DbPool<'_>, person_id: PersonId) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    certificates::table
      .filter(certificates::person_id.eq(person_id))
      .load::<Self>(conn)
      .await
      .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }

  pub async fn delete_by_person_id(pool: &mut DbPool<'_>, person_id: PersonId) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(certificates::table.filter(certificates::person_id.eq(person_id)))
      .execute(conn)
      .await
      .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }

  pub async fn delete_by_id_and_person(pool: &mut DbPool<'_>, certificate_id: i32, person_id: PersonId) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      certificates::table
        .filter(certificates::id.eq(certificate_id))
        .filter(certificates::person_id.eq(person_id))
    )
    .execute(conn)
    .await
    .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }

  pub async fn update_by_id_and_person(
    pool: &mut DbPool<'_>, 
    certificate_id: i32, 
    person_id: PersonId, 
    name: String,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(
      certificates::table
        .filter(certificates::id.eq(certificate_id))
        .filter(certificates::person_id.eq(person_id))
    )
    .set((
      certificates::name.eq(name),
      certificates::updated_at.eq(diesel::dsl::now),
    ))
    .get_result::<Self>(conn)
    .await
    .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }
}