use crate::{
  newtypes::{LanguageProfileId, PersonId},
  source::language_profile::{
    LanguageProfile, LanguageProfileInsertForm, LanguageProfileResponse,
    LanguageProfileUpdateForm,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::language_profile;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Crud for LanguageProfile {
  type InsertForm = LanguageProfileInsertForm;
  type UpdateForm = LanguageProfileUpdateForm;
  type IdType = LanguageProfileId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(language_profile::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    let mut update_form = form.clone();
    update_form.updated_at = Some(Utc::now());
    
    diesel::update(language_profile::table.find(id))
      .set(&update_form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }
}

impl LanguageProfile {
  pub async fn list_for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> FastJobResult<Vec<LanguageProfileResponse>> {
    let conn = &mut get_conn(pool).await?;

    let language_profiles: Vec<LanguageProfile> = language_profile::table
      .filter(language_profile::person_id.eq(person_id))
      .load(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;

    Ok(
      language_profiles
        .into_iter()
        .map(LanguageProfileResponse::from)
        .collect(),
    )
  }

  pub async fn save_language_profiles(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    profiles: Vec<crate::source::language_profile::LanguageProfileRequest>,
  ) -> FastJobResult<Vec<LanguageProfileResponse>> {
    let mut results = Vec::new();

    for profile_request in profiles {
      let result = match profile_request.id {
        Some(id) => {
          // Update existing profile
          let update_form = LanguageProfileUpdateForm {
            lang: Some(profile_request.lang),
            level_name: Some(profile_request.level_name),
            updated_at: Some(Utc::now()),
          };
          Self::update(pool, id, &update_form).await?
        }
        None => {
          // Create new profile
          let insert_form = LanguageProfileInsertForm::new(
            person_id,
            profile_request.lang,
            profile_request.level_name,
          );
          Self::create(pool, &insert_form).await?
        }
      };
      results.push(LanguageProfileResponse::from(result));
    }

    Ok(results)
  }

}