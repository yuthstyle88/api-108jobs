#[cfg(feature = "full")]
use crate::{
  newtypes::{EducationId, PersonId},
  source::education::{Education, EducationInsertForm, EducationUpdateForm, EducationItem, EducationResponse},
  utils::{get_conn, DbPool},
};

use crate::traits::Crud;
use chrono::Utc;
#[cfg(feature = "full")]
use diesel::{
  ExpressionMethods,
  QueryDsl,
  BoolExpressionMethods,
};
use diesel::dsl::{insert_into, not};
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;
use diesel_async::scoped_futures::ScopedFutureExt;
use lemmy_db_schema_file::schema::education;
#[cfg(feature = "full")]
use lemmy_utils::error::{FastJobErrorExt, FastJobResult};
use lemmy_utils::error::FastJobErrorType;

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

  pub async fn delete_not_in_list(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    education_ids: &[EducationId],
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    
    diesel::delete(education::table)
      .filter(education::person_id.eq(person_id))
      .filter(education::id.ne_all(education_ids))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::Deleted)
  }

  pub async fn save_education_list(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    educations: &[EducationItem],
  ) -> FastJobResult<Vec<EducationResponse>> {
    let conn = &mut get_conn(pool).await?;
    conn.build_transaction().run(|conn| {
      async move {
        let entries: Vec<(EducationInsertForm, (String, String))> = educations
        .iter()
        .filter_map(|i| i.school_name.as_ref().zip(i.major.as_ref())
        .map(|(school, major)| (EducationInsertForm::new(person_id, school.clone(), major.clone()), (school.clone(), major.clone()))))
        .collect();
        let (forms, keys_to_keep): (Vec<_>, Vec<_>) = entries.into_iter().unzip();

        if forms.is_empty() {
          diesel::delete(education::table.filter(education::person_id.eq(person_id)))
          .execute(conn).await
          .with_fastjob_type(FastJobErrorType::DatabaseError)?;
          return Ok(Vec::new());
        }

        let upserted = insert_into(education::table)
        .values(&forms)
            .on_conflict((education::person_id, education::school_name, education::major ))
        .do_update()
        .set(education::updated_at.eq(Utc::now()))
        .returning(education::all_columns)
        .get_results::<Education>(conn).await
        .with_fastjob_type(FastJobErrorType::DatabaseError)?;

        let schools_to_keep: Vec<String> = keys_to_keep.iter().map(|(school, _)| school.clone()).collect();
        let majors_to_keep: Vec<String> = keys_to_keep.iter().map(|(_, major)| major.clone()).collect();

        diesel::delete(
          education::table.filter(
            education::person_id.eq(person_id)
            .and(not(education::school_name.eq_any(&schools_to_keep).and(education::major.eq_any(&majors_to_keep)))),
          ),
        )
        .execute(conn).await
        .with_fastjob_type(FastJobErrorType::DatabaseError)?;

        Ok(upserted.into_iter().map(EducationResponse::from).collect())
      }.scope_boxed()
    }).await
  }
}