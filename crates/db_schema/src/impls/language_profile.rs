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
use diesel::{ExpressionMethods, QueryDsl, OptionalExtension, BoolExpressionMethods};
use diesel::dsl::{insert_into, not};
use diesel::upsert::excluded;
use diesel_async::RunQueryDsl;
use diesel_async::scoped_futures::ScopedFutureExt;
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
        .with_fastjob_type(FastJobErrorType::Deleted)
  }

  pub async fn save_language_profile_list(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    language_profiles: &[LanguageProfileItem],
  ) -> FastJobResult<Vec<LanguageProfileResponse>> {
    let conn = &mut get_conn(pool).await?;
    conn.build_transaction().run(|conn| {
      async move {
        let entries: Vec<(LanguageProfileInsertForm, String)> = language_profiles
        .iter()
        .filter_map(|i| i.lang.as_ref().zip(i.level_id.as_ref())
        .filter(|(_, level)| **level >= 1 && **level <= 3)
        .map(|(lang, level_id)| (LanguageProfileInsertForm::new(person_id, lang.clone(), *level_id), lang.clone())))
        .collect();
        let (forms, langs_to_keep): (Vec<_>, Vec<_>) = entries.into_iter().unzip();

        if forms.is_empty() {
          diesel::delete(language_profile::table.filter(language_profile::person_id.eq(person_id)))
          .execute(conn).await
          .with_fastjob_type(FastJobErrorType::DatabaseError)?;
          return Ok(Vec::new());
        }

        let upserted = insert_into(language_profile::table)
        .values(&forms)
        .on_conflict((language_profile::person_id, language_profile::lang))
        .do_update()
        .set((
          language_profile::level_id.eq(excluded(language_profile::level_id)),
          language_profile::updated_at.eq(Utc::now()),
        ))
        .returning(language_profile::all_columns)
        .get_results::<LanguageProfile>(conn).await
        .with_fastjob_type(FastJobErrorType::DatabaseError)?;

        diesel::delete(
          language_profile::table.filter(
            language_profile::person_id.eq(person_id)
            .and(not(language_profile::lang.eq_any(&langs_to_keep))),
          ),
        )
        .execute(conn).await
        .with_fastjob_type(FastJobErrorType::DatabaseError)?;

        Ok(upserted.into_iter().map(LanguageProfileResponse::from).collect())
      }.scope_boxed()
    }).await
  }
}