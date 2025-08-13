use crate::traits::Crud;
#[cfg(feature = "full")]
use crate::{
  newtypes::{PersonId, SkillId},
  source::skills::{Skills, SkillsInsertForm, SkillsUpdateForm, SkillItem, SkillResponse},
  utils::{get_conn, DbPool},
};
#[cfg(feature = "full")]
use diesel::{ExpressionMethods, QueryDsl};
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
  pub async fn read_by_person_id(pool: &mut DbPool<'_>, person_id: PersonId) -> FastJobResult<Vec<Self>> {
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

    conn.build_transaction()
      .run(|conn| {
        Box::pin(async move {
          let mut saved_skills = Vec::new();
          let mut skill_ids = Vec::new();

          for skill_item in skills {
            match skill_item.id {
              Some(id) => {
                // Update existing skill
                let form = SkillsUpdateForm {
                  skill_name: skill_item.skill_name.clone(),
                  level_id: skill_item.level_id,
                };
                let updated = Self::update(&mut conn.into(), id, &form).await?;
                skill_ids.push(id);
                saved_skills.push(updated);
              }
              None => {
                // Create new skill
                if let (Some(skill_name), Some(level_id)) = (&skill_item.skill_name, skill_item.level_id) {
                  // Validate skill level
                  if level_id < 1 || level_id > 5 {
                    return Err(FastJobErrorType::InvalidField("Proficient level must from 1 to 5".to_string()).into());
                  }
                  
                  let form = SkillsInsertForm::new(
                    person_id,
                    skill_name.clone(),
                    Some(level_id),
                  );
                  let created = Self::create(&mut conn.into(), &form).await?;
                  skill_ids.push(created.id);
                  saved_skills.push(created);
                }
              }
            }
          }

          // Delete any records not in the current list
          Self::delete_not_in_list(&mut conn.into(), person_id, &skill_ids).await?;

          // Convert to response format
          let skill_responses: Vec<SkillResponse> = saved_skills
            .into_iter()
            .map(SkillResponse::from)
            .collect();

          Ok(skill_responses)
        }) as _
      })
      .await
  }
}
