#[cfg(feature = "full")]
use crate::{
  newtypes::WorkExperienceId,
  source::work_experience::{WorkExperience, WorkExperienceInsertForm, WorkExperienceUpdateForm},
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
}