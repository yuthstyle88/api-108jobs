use crate::traits::Crud;
#[cfg(feature = "full")]
use crate::{
  newtypes::{PersonId, SkillId},
  source::skills::{SkillItem, SkillResponse, Skills, SkillsInsertForm, SkillsUpdateForm},
  utils::{get_conn, DbPool},
};
use diesel::dsl::{insert_into, not};
use diesel::upsert::excluded;
#[cfg(feature = "full")]
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::scoped_futures::ScopedFutureExt;
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::skills;
#[cfg(feature = "full")]
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

#[cfg(feature = "full")]
impl Crud for Skills {
  type InsertForm = SkillsInsertForm;
  type UpdateForm = SkillsUpdateForm;
  type IdType = SkillId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(skills::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateSkill)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(skills::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateSkill)
  }
}

#[cfg(feature = "full")]
impl Skills {
  pub async fn read_by_person_id(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    skills::table
      .filter(skills::person_id.eq(person_id))
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn delete_by_id_and_person(
    pool: &mut DbPool<'_>,
    skill_id: SkillId,
    person_id: PersonId,
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      skills::table
        .filter(skills::id.eq(skill_id))
        .filter(skills::person_id.eq(person_id)),
    )
    .execute(conn)
    .await
    .with_fastjob_type(FastJobErrorType::Deleted)
  }

  pub async fn delete_not_in_list(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    skill_ids: &[SkillId],
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;

    diesel::delete(skills::table)
      .filter(skills::person_id.eq(person_id))
      .filter(skills::id.ne_all(skill_ids))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::Deleted)
  }

  pub async fn save_skills_list(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    skills: &[SkillItem],
  ) -> FastJobResult<Vec<SkillResponse>> {
    let conn = &mut get_conn(pool).await?;
    conn
      .build_transaction()
      .run(|conn| {
        async move {
          let entries: Vec<(SkillsInsertForm, String)> = skills
            .iter()
            .filter_map(|i| {
              i.skill_name
                .as_ref()
                .zip(i.level_id.as_ref())
                .filter(|(_, level)| **level >= 1 && **level <= 3)
                .map(|(skill_name, level_id)| {
                  (
                    SkillsInsertForm::new(person_id, skill_name.clone(), Some(*level_id)),
                    skill_name.clone(),
                  )
                })
            })
            .collect();
          let (forms, skills_to_keep): (Vec<_>, Vec<_>) = entries.into_iter().unzip();

          if forms.is_empty() {
            diesel::delete(skills::table.filter(skills::person_id.eq(person_id)))
              .execute(conn)
              .await
              .with_fastjob_type(FastJobErrorType::DatabaseError)?;
            return Ok(Vec::new());
          }

          let upserted = insert_into(skills::table)
            .values(&forms)
            .on_conflict((skills::person_id, skills::skill_name))
            .do_update()
            .set(skills::level_id.eq(excluded(skills::level_id)))
            .returning(skills::all_columns)
            .get_results::<Skills>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::DatabaseError)?;

          diesel::delete(
            skills::table.filter(
              skills::person_id
                .eq(person_id)
                .and(not(skills::skill_name.eq_any(&skills_to_keep))),
            ),
          )
          .execute(conn)
          .await
          .with_fastjob_type(FastJobErrorType::DatabaseError)?;

          Ok(upserted.into_iter().map(SkillResponse::from).collect())
        }
        .scope_boxed()
      })
      .await
  }
}
