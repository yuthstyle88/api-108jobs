#[cfg(feature = "full")]
use crate::{
  newtypes::{EducationId, PersonId},
  source::education::{Education, EducationInsertForm, EducationUpdateForm, EducationItem, EducationResponse},
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
      .with_fastjob_type(FastJobErrorType::CouldntDeleteEducation)
  }

  pub async fn save_education_list(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    educations: &[EducationItem],
  ) -> FastJobResult<Vec<EducationResponse>> {
    let conn = &mut get_conn(pool).await?;

    conn.build_transaction()
      .run(|conn| {
        Box::pin(async move {
          let mut saved_educations = Vec::new();
          let mut education_ids = Vec::new();

          for education_item in educations {
            match education_item.id {
              Some(id) => {
                // Update existing education
                let form = EducationUpdateForm {
                  school_name: education_item.school_name.clone(),
                  major: education_item.major.clone(),
                };
                let updated = Self::update(&mut conn.into(), id, &form).await?;
                education_ids.push(id);
                saved_educations.push(updated);
              }
              None => {
                // Create new education
                if let (Some(school_name), Some(major)) = (&education_item.school_name, &education_item.major) {
                  let form = EducationInsertForm::new(
                    person_id,
                    school_name.clone(),
                    major.clone(),
                  );
                  let created = Self::create(&mut conn.into(), &form).await?;
                  education_ids.push(created.id);
                  saved_educations.push(created);
                }
              }
            }
          }

          // Delete any records not in the current list
          Self::delete_not_in_list(&mut conn.into(), person_id, &education_ids).await?;

          // Convert to response format
          let education_responses: Vec<EducationResponse> = saved_educations
            .into_iter()
            .map(EducationResponse::from)
            .collect();

          Ok(education_responses)
        }) as _
      })
      .await
  }
}