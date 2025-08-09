use crate::{
  newtypes::PersonId,
  source::work_experience::{WorkExperience, WorkExperienceInsertForm},
  utils::{get_conn, DbPool},
};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::work_experience;
use lemmy_utils::error::{FastJobErrorExt, FastJobResult};

impl WorkExperience {
  pub async fn create(pool: &mut DbPool<'_>, form: &WorkExperienceInsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(work_experience::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }

  pub async fn read_by_person_id(pool: &mut DbPool<'_>, person_id: PersonId) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    work_experience::table
      .filter(work_experience::person_id.eq(person_id))
      .load::<Self>(conn)
      .await
      .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }

  pub async fn delete_by_person_id(pool: &mut DbPool<'_>, person_id: PersonId) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(work_experience::table.filter(work_experience::person_id.eq(person_id)))
      .execute(conn)
      .await
      .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }

  pub async fn delete_by_id_and_person(pool: &mut DbPool<'_>, experience_id: i32, person_id: PersonId) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      work_experience::table
        .filter(work_experience::id.eq(experience_id))
        .filter(work_experience::person_id.eq(person_id))
    )
    .execute(conn)
    .await
    .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }

  pub async fn update_by_id_and_person(
    pool: &mut DbPool<'_>, 
    experience_id: i32, 
    person_id: PersonId, 
    company_name: String,
    position: String,
    start_month: Option<String>,
    start_year: Option<i32>,
    end_month: Option<String>,
    end_year: Option<i32>,
    is_current: Option<bool>,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(
      work_experience::table
        .filter(work_experience::id.eq(experience_id))
        .filter(work_experience::person_id.eq(person_id))
    )
    .set((
      work_experience::company_name.eq(company_name),
      work_experience::position.eq(position),
      work_experience::start_month.eq(start_month),
      work_experience::start_year.eq(start_year),
      work_experience::end_month.eq(end_month),
      work_experience::end_year.eq(end_year),
      work_experience::is_current.eq(is_current),
      work_experience::updated_at.eq(diesel::dsl::now),
    ))
    .get_result::<Self>(conn)
    .await
    .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }
}