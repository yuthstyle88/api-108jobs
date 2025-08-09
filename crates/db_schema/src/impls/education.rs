#[cfg(feature = "full")]
use crate::{
  newtypes::EducationId,
  source::education::{Education, EducationInsertForm, EducationUpdateForm},
  utils::{get_conn, DbPool},
};

use crate::traits::Crud;
#[cfg(feature = "full")]
use diesel::{
  ExpressionMethods,
  QueryDsl,
};
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::education;
#[cfg(feature = "full")]
use lemmy_utils::error::{FastJobErrorExt, FastJobResult};
use lemmy_utils::error::FastJobErrorType;
use crate::newtypes::PersonId;

#[cfg(feature = "full")]
impl Crud for Education {
  type InsertForm = EducationInsertForm;
  type UpdateForm = EducationUpdateForm;
  type IdType = EducationId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(education::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateEducation)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(education::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateEducation)
  }
}
impl Education {
  pub async fn read_by_person_id(pool: &mut DbPool<'_>, person_id: PersonId) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    education::table
    .filter(education::person_id.eq(person_id))
    .load::<Self>(conn)
    .await
    .with_fastjob_type(FastJobErrorType::NotFound)
  }
}