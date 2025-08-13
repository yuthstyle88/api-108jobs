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
use diesel::{ExpressionMethods, QueryDsl, OptionalExtension};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::language_profile;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use tracing::warn;

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

  pub async fn find_by_person_and_language(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    lang: &str,
  ) -> FastJobResult<Option<LanguageProfile>> {
    let conn = &mut get_conn(pool).await?;

    let result = language_profile::table
      .filter(language_profile::person_id.eq(person_id))
      .filter(language_profile::lang.eq(lang))
      .first::<LanguageProfile>(conn)
      .await
      .optional()
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;

    Ok(result)
  }

  pub async fn save_language_profiles(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    profiles: Vec<crate::source::language_profile::LanguageProfileRequest>,
  ) -> FastJobResult<Vec<LanguageProfileResponse>> {
    let mut results = Vec::new();

    for profile_request in profiles {
      match (profile_request.id, profile_request.deleted) {
        (Some(id), true) => {
          // Delete existing profile
          Self::delete(pool, id).await?;
          // Don't add to results for deleted items
        }
        (Some(id), false) => {
          // Update existing profile
          let update_form = LanguageProfileUpdateForm {
            lang: Some(profile_request.lang),
            level_name: Some(profile_request.level_name),
            updated_at: Some(Utc::now()),
          };
          let result = Self::update(pool, id, &update_form).await?;
          results.push(LanguageProfileResponse::from(result));
        }
        (None, false) => {
          // Create new profile or update existing one if it already exists
          if let (lang, level_name) = (profile_request.lang.clone(), profile_request.level_name) {
            // Check if a language profile already exists for this person and language
            if let Some(existing_profile) = Self::find_by_person_and_language(pool, person_id, &lang).await? {
              // Log the duplicate detection and update existing profile instead
              warn!(
                "Duplicate language profile detected for person_id: {}, lang: '{}'. Updating existing profile with id: {} instead of creating new one.",
                person_id.0,
                lang,
                existing_profile.id.0
              );
              
              let update_form = LanguageProfileUpdateForm {
                lang: Some(lang),
                level_name: Some(level_name),
                updated_at: Some(Utc::now()),
              };
              let result = Self::update(pool, existing_profile.id, &update_form).await?;
              results.push(LanguageProfileResponse::from(result));
            } else {
              // Create new profile only if none exists
              let insert_form = LanguageProfileInsertForm::new(
                person_id,
                lang,
                level_name,
              );
              let result = Self::create(pool, &insert_form).await?;
              results.push(LanguageProfileResponse::from(result));
            }
          }
          // Skip invalid create requests
        }
        (None, true) => {
          // Skip invalid delete requests (can't delete without ID)
        }
      }
    }

    Ok(results)
  }

}