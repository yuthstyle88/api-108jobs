#[cfg(feature = "full")]
use crate::{
  newtypes::WorkExperienceId,
  source::work_experience::{WorkExperience, WorkExperienceInsertForm, WorkExperienceUpdateForm, WorkExperienceItem, WorkExperienceResponse},
  utils::{get_conn, DbPool},
};
use crate::traits::Crud;
use crate::newtypes::PersonId;
#[cfg(feature = "full")]
use diesel::{ExpressionMethods, QueryDsl, BoolExpressionMethods};
use diesel::dsl::{insert_into, not};
use diesel::upsert::excluded;
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;
use diesel_async::scoped_futures::ScopedFutureExt;
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
      .with_fastjob_type(FastJobErrorType::Deleted)
  }

  pub async fn save_work_experience_list(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    work_experiences: &[WorkExperienceItem],
  ) -> FastJobResult<Vec<WorkExperienceResponse>> {
    let conn = &mut get_conn(pool).await?;
    conn.build_transaction().run(|conn| {
      async move {
        let entries: Vec<(WorkExperienceInsertForm, (String, String))> = work_experiences
        .iter()
        .filter_map(|i| i.company_name.as_ref().zip(i.position.as_ref()).zip(i.startmonth.as_ref()).zip(i.startyear.as_ref())
        .map(|(((company, position), start_month), start_year)| (WorkExperienceInsertForm::new(person_id, company.clone(), position.clone(), *start_month, *start_year, i.endmonth, i.endyear), (company.clone(), position.clone()))))
        .collect();
        let (forms, keys_to_keep): (Vec<_>, Vec<_>) = entries.into_iter().unzip();

        if forms.is_empty() {
          diesel::delete(work_experience::table.filter(work_experience::person_id.eq(person_id)))
          .execute(conn).await
          .with_fastjob_type(FastJobErrorType::DatabaseError)?;
          return Ok(Vec::new());
        }

        let upserted = insert_into(work_experience::table)
        .values(&forms)
        .on_conflict((work_experience::person_id, work_experience::company_name, work_experience::position))
        .do_update()
        .set((
          work_experience::startmonth.eq(excluded(work_experience::startmonth)),
          work_experience::startyear.eq(excluded(work_experience::startyear)),
          work_experience::endmonth.eq(excluded(work_experience::endmonth)),
          work_experience::endyear.eq(excluded(work_experience::endyear)),
          work_experience::is_current.eq(excluded(work_experience::is_current)),
        ))
        .returning(work_experience::all_columns)
        .get_results::<WorkExperience>(conn).await
        .with_fastjob_type(FastJobErrorType::DatabaseError)?;

        let companies_to_keep: Vec<String> = keys_to_keep.iter().map(|(company, _)| company.clone()).collect();
        let positions_to_keep: Vec<String> = keys_to_keep.iter().map(|(_, position)| position.clone()).collect();

        diesel::delete(
          work_experience::table.filter(
            work_experience::person_id.eq(person_id)
            .and(not(work_experience::company_name.eq_any(&companies_to_keep).and(work_experience::position.eq_any(&positions_to_keep)))),
          ),
        )
        .execute(conn).await
        .with_fastjob_type(FastJobErrorType::DatabaseError)?;

        Ok(upserted.into_iter().map(WorkExperienceResponse::from).collect())
      }.scope_boxed()
    }).await
  }
}