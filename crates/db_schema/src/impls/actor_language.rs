use crate::{
  diesel::JoinOnDsl,
  newtypes::{CategoryId, InstanceId, LanguageId, LocalUserId, SiteId},
  source::{
    actor_language::{
      CategoryLanguage,
      CategoryLanguageForm,
      LocalUserLanguage,
      LocalUserLanguageForm,
      SiteLanguage,
      SiteLanguageForm,
    },
    language::Language,
    site::Site,
  },
  utils::{get_conn, DbPool},
};
use diesel::{
  delete,
  dsl::{count, exists},
  insert_into,
  select,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncPgConnection, RunQueryDsl};
use app_108jobs_db_schema_file::schema::{
  category_language,
  local_site,
  local_user_language,
  site,
  site_language,
};
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use tokio::sync::OnceCell;

pub const UNDETERMINED_ID: LanguageId = LanguageId(0);

impl LocalUserLanguage {
  pub async fn read(
    pool: &mut DbPool<'_>,
    for_local_user_id: LocalUserId,
  ) -> FastJobResult<Vec<LanguageId>> {
    let conn = &mut get_conn(pool).await?;

    let langs = local_user_language::table
      .filter(local_user_language::local_user_id.eq(for_local_user_id))
      .order(local_user_language::language_id)
      .select(local_user_language::language_id)
      .get_results(conn)
      .await?;
    convert_read_languages(conn, langs).await
  }

  /// Update the user's languages.
  ///
  /// If no language_id vector is given, it will show all languages
  pub async fn update(
    pool: &mut DbPool<'_>,
    language_ids: Vec<LanguageId>,
    for_local_user_id: LocalUserId,
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    let lang_ids = convert_update_languages(conn, language_ids).await?;

    // No need to update if languages are unchanged
    let current = LocalUserLanguage::read(&mut conn.into(), for_local_user_id).await?;
    if current == lang_ids {
      return Ok(0);
    }

    conn
      .run_transaction(|conn| {
        async move {
          // Delete old languages, not including new languages
          delete(local_user_language::table)
            .filter(local_user_language::local_user_id.eq(for_local_user_id))
            .filter(local_user_language::language_id.ne_all(&lang_ids))
            .execute(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntUpdateLanguages)?;

          let forms = lang_ids
            .iter()
            .map(|&l| LocalUserLanguageForm {
              local_user_id: for_local_user_id,
              language_id: l,
            })
            .collect::<Vec<_>>();

          // Insert new languages
          insert_into(local_user_language::table)
            .values(forms)
            .on_conflict((
              local_user_language::language_id,
              local_user_language::local_user_id,
            ))
            .do_nothing()
            .execute(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntUpdateLanguages)
        }
        .scope_boxed()
      })
      .await
  }
}

impl SiteLanguage {
  pub async fn read_local_raw(pool: &mut DbPool<'_>) -> FastJobResult<Vec<LanguageId>> {
    let conn = &mut get_conn(pool).await?;
    site::table
      .inner_join(local_site::table)
      .inner_join(site_language::table)
      .order(site_language::language_id)
      .select(site_language::language_id)
      .load(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn read(pool: &mut DbPool<'_>, for_site_id: SiteId) -> FastJobResult<Vec<LanguageId>> {
    let conn = &mut get_conn(pool).await?;
    let langs = site_language::table
      .filter(site_language::site_id.eq(for_site_id))
      .order(site_language::language_id)
      .select(site_language::language_id)
      .load(conn)
      .await?;

    convert_read_languages(conn, langs).await
  }

  pub async fn update(
    pool: &mut DbPool<'_>,
    language_ids: Vec<LanguageId>,
    site: &Site,
  ) -> FastJobResult<()> {
    let conn = &mut get_conn(pool).await?;
    let for_site_id = site.id;
    let instance_id = site.instance_id;
    let lang_ids = convert_update_languages(conn, language_ids).await?;

    // No need to update if languages are unchanged
    let current = SiteLanguage::read(&mut conn.into(), site.id).await?;
    if current == lang_ids {
      return Ok(());
    }

    conn
      .run_transaction(|conn| {
        async move {
          // Delete old languages, not including new languages
          delete(site_language::table)
            .filter(site_language::site_id.eq(for_site_id))
            .filter(site_language::language_id.ne_all(&lang_ids))
            .execute(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntUpdateLanguages)?;

          let forms = lang_ids
            .iter()
            .map(|&l| SiteLanguageForm {
              site_id: for_site_id,
              language_id: l,
            })
            .collect::<Vec<_>>();

          // Insert new languages
          insert_into(site_language::table)
            .values(forms)
            .on_conflict((site_language::site_id, site_language::language_id))
            .do_nothing()
            .execute(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntUpdateLanguages)?;

          CategoryLanguage::limit_languages(conn, instance_id).await?;

          Ok(())
        }
        .scope_boxed()
      })
      .await
  }
}

impl CategoryLanguage {
  /// Returns true if the given language is one of configured languages for given category
  async fn is_allowed_category_language(
    pool: &mut DbPool<'_>,
    for_language_id: LanguageId,
    for_category_id: CategoryId,
  ) -> FastJobResult<()> {
    use app_108jobs_db_schema_file::schema::category_language::dsl::category_language;
    let conn = &mut get_conn(pool).await?;

    let is_allowed = select(exists(
      category_language.find((for_category_id, for_language_id)),
    ))
    .get_result(conn)
    .await?;

    if is_allowed {
      Ok(())
    } else {
      Err(FastJobErrorType::LanguageNotAllowed)?
    }
  }

  /// When site languages are updated, delete all languages of local communities which are not
  /// also part of site languages. This is because post/comment language is only checked against
  /// category language, and it shouldnt be possible to post content in languages which are not
  /// allowed by local site.
  async fn limit_languages(
    conn: &mut AsyncPgConnection,
    for_instance_id: InstanceId,
  ) -> FastJobResult<()> {
    use app_108jobs_db_schema_file::schema::{
      category::dsl as c,
      category_language::dsl as cl,
      site_language::dsl as sl,
    };
    let category_languages: Vec<LanguageId> = cl::category_language
      .left_outer_join(sl::site_language.on(cl::language_id.eq(sl::language_id)))
      .inner_join(c::category)
      .filter(c::instance_id.eq(for_instance_id))
      .filter(sl::language_id.is_null())
      .select(cl::language_id)
      .get_results(conn)
      .await?;

    for c in category_languages {
      delete(cl::category_language.filter(cl::language_id.eq(c)))
        .execute(conn)
        .await?;
    }
    Ok(())
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    for_category_id: CategoryId,
  ) -> FastJobResult<Vec<LanguageId>> {
    use app_108jobs_db_schema_file::schema::category_language::dsl::{
      category_id,
      category_language,
      language_id,
    };
    let conn = &mut get_conn(pool).await?;
    let langs = category_language
      .filter(category_id.eq(for_category_id))
      .order(language_id)
      .select(language_id)
      .get_results(conn)
      .await?;
    convert_read_languages(conn, langs).await
  }

  pub async fn update(
    pool: &mut DbPool<'_>,
    mut language_ids: Vec<LanguageId>,
    for_category_id: CategoryId,
  ) -> FastJobResult<usize> {
    if language_ids.is_empty() {
      language_ids = SiteLanguage::read_local_raw(pool).await?;
    }
    let conn = &mut get_conn(pool).await?;
    let lang_ids = convert_update_languages(conn, language_ids).await?;

    // No need to update if languages are unchanged
    let current = CategoryLanguage::read(&mut conn.into(), for_category_id).await?;
    if current == lang_ids {
      return Ok(0);
    }

    let form = lang_ids
      .iter()
      .map(|&language_id| CategoryLanguageForm {
        category_id: for_category_id,
        language_id,
      })
      .collect::<Vec<_>>();

    conn
      .run_transaction(|conn| {
        async move {
          // Delete old languages, not including new languages
          delete(category_language::table)
            .filter(category_language::category_id.eq(for_category_id))
            .filter(category_language::language_id.ne_all(&lang_ids))
            .execute(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntUpdateLanguages)?;

          // Insert new languages
          insert_into(category_language::table)
            .values(form)
            .on_conflict((
              category_language::category_id,
              category_language::language_id,
            ))
            .do_nothing()
            .execute(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntUpdateLanguages)
        }
        .scope_boxed()
      })
      .await
  }
}

pub async fn validate_post_language(
  pool: &mut DbPool<'_>,
  language_id: Option<LanguageId>,
  category_id: CategoryId,
  local_user_id: LocalUserId,
) -> FastJobResult<LanguageId> {
  use app_108jobs_db_schema_file::schema::{
    category_language::dsl as cl,
    local_user_language::dsl as ul,
  };
  let conn = &mut get_conn(pool).await?;
  let language_id = match language_id {
    None | Some(LanguageId(0)) => {
      let mut intersection = ul::local_user_language
        .inner_join(cl::category_language.on(ul::language_id.eq(cl::language_id)))
        .filter(ul::local_user_id.eq(local_user_id))
        .filter(cl::category_id.eq(category_id))
        .select(cl::language_id)
        .get_results::<LanguageId>(conn)
        .await?;

      if intersection.len() == 1 {
        intersection.pop().unwrap_or(UNDETERMINED_ID)
      } else if intersection.len() == 2 && intersection.contains(&UNDETERMINED_ID) {
        intersection.retain(|i| i != &UNDETERMINED_ID);
        intersection.pop().unwrap_or(UNDETERMINED_ID)
      } else {
        UNDETERMINED_ID
      }
    }
    Some(lid) => lid,
  };

  CategoryLanguage::is_allowed_category_language(pool, language_id, category_id).await?;
  Ok(language_id)
}

/// If no language is given, set all languages
async fn convert_update_languages(
  conn: &mut AsyncPgConnection,
  language_ids: Vec<LanguageId>,
) -> FastJobResult<Vec<LanguageId>> {
  if language_ids.is_empty() {
    Ok(
      Language::read_all(&mut conn.into())
        .await?
        .into_iter()
        .map(|l| l.id)
        .collect(),
    )
  } else {
    Ok(language_ids)
  }
}

/// If all languages are returned, return empty vec instead
#[allow(clippy::expect_used)]
async fn convert_read_languages(
  conn: &mut AsyncPgConnection,
  language_ids: Vec<LanguageId>,
) -> FastJobResult<Vec<LanguageId>> {
  static ALL_LANGUAGES_COUNT: OnceCell<i64> = OnceCell::const_new();
  let count: usize = (*ALL_LANGUAGES_COUNT
    .get_or_init(|| async {
      use app_108jobs_db_schema_file::schema::language::dsl::{id, language};
      let count: i64 = language
        .select(count(id))
        .first(conn)
        .await
        .expect("read number of languages");
      count
    })
    .await)
    .try_into()?;

  if language_ids.len() == count {
    Ok(vec![])
  } else {
    Ok(language_ids)
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use super::*;
  use crate::{
    source::{
        category::{Category, CategoryInsertForm},
        local_site::LocalSite,
        local_user::{LocalUser, LocalUserInsertForm},
        person::{Person, PersonInsertForm},
    },
    test_data::TestData,
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  async fn test_langs1(pool: &mut DbPool<'_>) -> FastJobResult<Vec<LanguageId>> {
    Ok(vec![
      Language::read_id_from_code(pool, "en").await?,
      Language::read_id_from_code(pool, "fr").await?,
      Language::read_id_from_code(pool, "ru").await?,
    ])
  }
  async fn test_langs2(pool: &mut DbPool<'_>) -> FastJobResult<Vec<LanguageId>> {
    Ok(vec![
      Language::read_id_from_code(pool, "fi").await?,
      Language::read_id_from_code(pool, "se").await?,
    ])
  }

  #[tokio::test]
  #[serial]
  async fn test_convert_update_languages() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    // call with empty vec, returns all languages
    let conn = &mut get_conn(pool).await?;
    let converted1 = convert_update_languages(conn, vec![]).await?;
    assert_eq!(184, converted1.len());

    // call with nonempty vec, returns same vec
    let test_langs = test_langs1(&mut conn.into()).await?;
    let converted2 = convert_update_languages(conn, test_langs.clone()).await?;
    assert_eq!(test_langs, converted2);

    Ok(())
  }
  #[tokio::test]
  #[serial]
  async fn test_convert_read_languages() -> FastJobResult<()> {
    use app_108jobs_db_schema_file::schema::language::dsl::{id, language};
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    // call with all languages, returns empty vec
    let conn = &mut get_conn(pool).await?;
    let all_langs = language.select(id).get_results(conn).await?;
    let converted1: Vec<LanguageId> = convert_read_languages(conn, all_langs).await?;
    assert_eq!(0, converted1.len());

    // call with nonempty vec, returns same vec
    let test_langs = test_langs1(&mut conn.into()).await?;
    let converted2 = convert_read_languages(conn, test_langs.clone()).await?;
    assert_eq!(test_langs, converted2);

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_site_languages() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let data = TestData::create(pool).await?;
    let site_languages1 = SiteLanguage::read_local_raw(pool).await?;
    // site is created with all languages
    assert_eq!(184, site_languages1.len());

    let test_langs = test_langs1(pool).await?;
    SiteLanguage::update(pool, test_langs.clone(), &data.site).await?;

    let site_languages2 = SiteLanguage::read_local_raw(pool).await?;
    // after update, site only has new languages
    assert_eq!(test_langs, site_languages2);

    data.delete(pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_user_languages() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let data = TestData::create(pool).await?;

    let person_form = PersonInsertForm::test_form(data.instance.id, "my test person");
    let person = Person::create(pool, &person_form).await?;
    let local_user_form = LocalUserInsertForm::test_form(person.id);

    let local_user = LocalUser::create(pool, &local_user_form, vec![]).await?;
    let local_user_langs1 = LocalUserLanguage::read(pool, local_user.id).await?;

    // new user should be initialized with all languages
    assert_eq!(0, local_user_langs1.len());

    // update user languages
    let test_langs2 = test_langs2(pool).await?;
    LocalUserLanguage::update(pool, test_langs2, local_user.id).await?;
    let local_user_langs2 = LocalUserLanguage::read(pool, local_user.id).await?;
    assert_eq!(2, local_user_langs2.len());

    Person::delete(pool, person.id).await?;
    LocalUser::delete(pool, local_user.id).await?;
    LocalSite::delete(pool).await?;
    data.delete(pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_category_languages() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = TestData::create(pool).await?;
    let test_langs = test_langs1(pool).await?;
    SiteLanguage::update(pool, test_langs.clone(), &data.site).await?;

    let read_site_langs = SiteLanguage::read(pool, data.site.id).await?;
    assert_eq!(test_langs, read_site_langs);

    // Test the local ones are the same
    let read_local_site_langs = SiteLanguage::read_local_raw(pool).await?;
    assert_eq!(test_langs, read_local_site_langs);

    let category_form = CategoryInsertForm::new(
      data.instance.id,
      "test category".to_string(),
      "test category".to_string(),
    );
    let category = Category::create(pool, &category_form).await?;
    let category_langs1 = CategoryLanguage::read(pool, category.id).await?;

    // category is initialized with site languages
    assert_eq!(test_langs, category_langs1);

    let allowed_lang1 =
      CategoryLanguage::is_allowed_category_language(pool, test_langs[0], category.id).await;
    assert!(allowed_lang1.is_ok());

    let test_langs2 = test_langs2(pool).await?;
    let allowed_lang2 =
      CategoryLanguage::is_allowed_category_language(pool, test_langs2[0], category.id).await;
    assert!(allowed_lang2.is_err());

    // limit site languages to en, fi. after this, category languages should be updated to
    // intersection of old languages (en, fr, ru) and (en, fi), which is only fi.
    SiteLanguage::update(pool, vec![test_langs[0], test_langs2[0]], &data.site).await?;
    let category_langs2 = CategoryLanguage::read(pool, category.id).await?;
    assert_eq!(vec![test_langs[0]], category_langs2);

    // update category languages to different ones
    CategoryLanguage::update(pool, test_langs2.clone(), category.id).await?;
    let category_langs3 = CategoryLanguage::read(pool, category.id).await?;
    assert_eq!(test_langs2, category_langs3);

    Category::delete(pool, category.id).await?;
    LocalSite::delete(pool).await?;
    data.delete(pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_validate_post_language() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = TestData::create(pool).await?;
    let test_langs = test_langs1(pool).await?;
    let test_langs2 = test_langs2(pool).await?;

    let category_form = CategoryInsertForm::new(
      data.instance.id,
      "test category".to_string(),
      "test category".to_string(),
    );
    let category = Category::create(pool, &category_form).await?;
    CategoryLanguage::update(pool, test_langs, category.id).await?;

    let person_form = PersonInsertForm::test_form(data.instance.id, "my test person");
    let person = Person::create(pool, &person_form).await?;
    let local_user_form = LocalUserInsertForm::test_form(person.id);
    let local_user = LocalUser::create(pool, &local_user_form, vec![]).await?;
    LocalUserLanguage::update(pool, test_langs2, local_user.id).await?;

    // no overlap in user/category languages, so defaults to undetermined
    let def1 = validate_post_language(pool, None, category.id, local_user.id).await;
    assert_eq!(
      Some(FastJobErrorType::LanguageNotAllowed),
      def1.err().map(|e| e.error_type)
    );

    let ru = Language::read_id_from_code(pool, "ru").await?;
    let test_langs3 = vec![
      ru,
      Language::read_id_from_code(pool, "fi").await?,
      Language::read_id_from_code(pool, "se").await?,
      UNDETERMINED_ID,
    ];
    LocalUserLanguage::update(pool, test_langs3, local_user.id).await?;

    // this time, both have ru as common lang
    let def2 = validate_post_language(pool, None, category.id, local_user.id).await?;
    assert_eq!(ru, def2);

    Person::delete(pool, person.id).await?;
    Category::delete(pool, category.id).await?;
    LocalUser::delete(pool, local_user.id).await?;
    LocalSite::delete(pool).await?;
    data.delete(pool).await?;

    Ok(())
  }
}
