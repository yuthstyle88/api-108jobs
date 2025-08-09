#[cfg(feature = "full")]
use crate::{
  newtypes::PersonId,
  source::education::{Education, EducationInsertForm},
  utils::{get_conn, DbPool},
};

#[cfg(feature = "full")]
use diesel::{ExpressionMethods, QueryDsl};
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::education;
#[cfg(feature = "full")]
use lemmy_utils::error::{FastJobErrorExt, FastJobResult};

#[cfg(feature = "full")]
impl Education {
  pub async fn create(pool: &mut DbPool<'_>, form: &EducationInsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(education::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }

  pub async fn read_by_person_id(pool: &mut DbPool<'_>, person_id: PersonId) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    education::table
      .filter(education::person_id.eq(person_id))
      .load::<Self>(conn)
      .await
      .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }

  pub async fn delete_by_person_id(pool: &mut DbPool<'_>, person_id: PersonId) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(education::table.filter(education::person_id.eq(person_id)))
      .execute(conn)
      .await
      .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }

  pub async fn delete_by_id_and_person(pool: &mut DbPool<'_>, education_id: i32, person_id: PersonId) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      education::table
        .filter(education::id.eq(education_id))
        .filter(education::person_id.eq(person_id))
    )
    .execute(conn)
    .await
    .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }

  pub async fn update_by_id_and_person(
    pool: &mut DbPool<'_>, 
    education_id: i32, 
    person_id: PersonId, 
    school_name: String, 
    major: String
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(
      education::table
        .filter(education::id.eq(education_id))
        .filter(education::person_id.eq(person_id))
    )
    .set((
      education::school_name.eq(school_name),
      education::major.eq(major),
      education::updated_at.eq(diesel::dsl::now),
    ))
    .get_result::<Self>(conn)
    .await
    .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }
}