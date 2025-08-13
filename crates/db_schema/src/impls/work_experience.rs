#[cfg(feature = "full")]
use crate::{
  newtypes::WorkExperienceId,
  source::work_experience::{WorkExperience, WorkExperienceInsertForm, WorkExperienceUpdateForm, WorkExperienceItem, WorkExperienceResponse},
  utils::{get_conn, DbPool},
};
use crate::traits::Crud;
use crate::newtypes::PersonId;
#[cfg(feature = "full")]
use diesel::{ExpressionMethods, QueryDsl};
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::work_experience;
#[cfg(feature = "full")]
use lemmy_utils::error::{FastJobErrorExt, FastJobResult};
use lemmy_utils::error::FastJobErrorType;

#[cfg(feature = "full")]
impl Crud for WorkExperience {
  type InsertForm = WorkExperienceInsertForm;
  type UpdateForm = WorkExperienceUpdateForm;
  type IdType = WorkExperienceId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(work_experience::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateWorkExperience)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(work_experience::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateWorkExperience)
  }
}

impl WorkExperience {
  pub async fn read_by_person_id(pool: &mut DbPool<'_>, person_id: PersonId) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    work_experience::table
      .filter(work_experience::person_id.eq(person_id))
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn delete_not_in_list(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    work_experience_ids: &[WorkExperienceId],
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    
    diesel::delete(work_experience::table)
      .filter(work_experience::person_id.eq(person_id))
      .filter(work_experience::id.ne_all(work_experience_ids))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntDeleteWorkExperience)
  }

  pub async fn save_work_experience_list(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    work_experiences: &[WorkExperienceItem],
  ) -> FastJobResult<Vec<WorkExperienceResponse>> {
    let conn = &mut get_conn(pool).await?;

    conn.build_transaction()
      .run(|conn| {
        Box::pin(async move {
          let mut saved_work_experiences = Vec::new();
          let mut work_experience_ids = Vec::new();

          for work_exp_item in work_experiences {
            match work_exp_item.id {
              Some(id) => {
                // Update existing work experience
                let form = WorkExperienceUpdateForm {
                  company_name: work_exp_item.company_name.clone(),
                  position: work_exp_item.position.clone(),
                  start_date: work_exp_item.start_date,
                  end_date: Some(work_exp_item.end_date),
                  is_current: Some(work_exp_item.is_current),
                };
                let updated = Self::update(&mut conn.into(), id, &form).await?;
                work_experience_ids.push(id);
                saved_work_experiences.push(updated);
              }
              None => {
                // Create new work experience
                if let (Some(company_name), Some(position), Some(start_date)) = 
                  (&work_exp_item.company_name, &work_exp_item.position, work_exp_item.start_date) {
                  let form = WorkExperienceInsertForm::new(
                    person_id,
                    company_name.clone(),
                    position.clone(),
                    start_date,
                    work_exp_item.end_date,
                  );
                  let created = Self::create(&mut conn.into(), &form).await?;
                  work_experience_ids.push(created.id);
                  saved_work_experiences.push(created);
                }
              }
            }
          }

          // Delete any records not in the current list
          Self::delete_not_in_list(&mut conn.into(), person_id, &work_experience_ids).await?;

          // Convert to response format
          let work_experience_responses: Vec<WorkExperienceResponse> = saved_work_experiences
            .into_iter()
            .map(WorkExperienceResponse::from)
            .collect();

          Ok(work_experience_responses)
        }) as _
      })
      .await
  }
}