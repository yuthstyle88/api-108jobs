use crate::{
  newtypes::{LanguageProfileId, PersonId},
  source::language_profile::{
    LanguageProfile, LanguageProfileInsertForm, LanguageProfileResponse,
    LanguageProfileUpdateForm, LanguageProfileItem
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, OptionalExtension};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{language_profile};
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


  pub async fn delete_not_in_list(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    language_profile_ids: &[LanguageProfileId],
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;

    diesel::delete(language_profile::table)
        .filter(language_profile::person_id.eq(person_id))
        .filter(language_profile::id.ne_all(language_profile_ids))
        .execute(conn)
        .await
        .with_fastjob_type(FastJobErrorType::CouldntDeleteLanguageProfile)
  }

  pub async fn save_language_profile_list(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    language_profiles: &[LanguageProfileItem],
  ) -> FastJobResult<Vec<LanguageProfileResponse>> {
    let conn = &mut get_conn(pool).await?;

    conn.build_transaction()
        .run(|conn| {
          Box::pin(async move {
            let mut saved_language_profiles = Vec::new();
            let mut language_profile_ids = Vec::new();

            for lang_profile_item in language_profiles {
              match lang_profile_item.id {
                Some(id) => {
                  // Update existing work experience
                  let form = LanguageProfileUpdateForm {
                    lang: lang_profile_item.lang.clone(),
                    level_name: lang_profile_item.level_name.clone(),
                    updated_at: Some(Utc::now()),
                  };
                  let updated = Self::update(&mut conn.into(), id, &form).await?;
                  language_profile_ids.push(id);
                  saved_language_profiles.push(updated);
                }
                None => {
                  // Create new work experience
                  if let (Some(ref lang), Some(ref level_name)) =
                      (&lang_profile_item.lang, &lang_profile_item.level_name) {
                    let form = LanguageProfileInsertForm::new(
                      person_id,
                      lang.clone(),
                      level_name.clone()
                    );
                    let created = Self::create(&mut conn.into(), &form).await?;
                    language_profile_ids.push(created.id);
                    saved_language_profiles.push(created);
                  }
                }
              }
            }

            // Delete any records not in the current list
            Self::delete_not_in_list(&mut conn.into(), person_id, &language_profile_ids).await?;

            // Convert to response format
            let language_profile_responses: Vec<LanguageProfileResponse> = saved_language_profiles
                .into_iter()
                .map(LanguageProfileResponse::from)
                .collect();

            Ok(language_profile_responses)
          }) as _
        })
        .await
  }
}