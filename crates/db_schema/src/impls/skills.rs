use crate::{
  newtypes::PersonId,
  source::skills::{Skills, SkillsInsertForm},
  utils::{get_conn, DbPool},
};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::skills;
use lemmy_utils::error::{FastJobErrorExt, FastJobResult};

impl Skills {
  pub async fn create(pool: &mut DbPool<'_>, form: &SkillsInsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(skills::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }

  pub async fn read_by_person_id(pool: &mut DbPool<'_>, person_id: PersonId) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    skills::table
      .filter(skills::person_id.eq(person_id))
      .load::<Self>(conn)
      .await
      .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }

  pub async fn delete_by_person_id(pool: &mut DbPool<'_>, person_id: PersonId) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(skills::table.filter(skills::person_id.eq(person_id)))
      .execute(conn)
      .await
      .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }

  pub async fn delete_by_id_and_person(pool: &mut DbPool<'_>, skill_id: i32, person_id: PersonId) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      skills::table
        .filter(skills::id.eq(skill_id))
        .filter(skills::person_id.eq(person_id))
    )
    .execute(conn)
    .await
    .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }

  pub async fn update_by_id_and_person(
    pool: &mut DbPool<'_>, 
    skill_id: i32, 
    person_id: PersonId, 
    skill_name: String,
    level_id: Option<i32>,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(
      skills::table
        .filter(skills::id.eq(skill_id))
        .filter(skills::person_id.eq(person_id))
    )
    .set((
      skills::skill_name.eq(skill_name),
      skills::level_id.eq(level_id),
      skills::updated_at.eq(diesel::dsl::now),
    ))
    .get_result::<Self>(conn)
    .await
    .with_fastjob_type(lemmy_utils::error::FastJobErrorType::DatabaseError)
  }
}