use super::actor_language::UNDETERMINED_ID;
use crate::{
  diesel::ExpressionMethods,
  newtypes::LanguageId,
  schema::language,
  source::language::Language,
  utils::{get_conn, DbPool},
};
use app_108jobs_core::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;

impl Language {
  pub async fn read_all(pool: &mut DbPool<'_>) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    language::table
      .load(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn read_from_id(pool: &mut DbPool<'_>, id_: LanguageId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    language::table
      .find(id_)
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  /// Attempts to find the given language code and return its ID.
  pub async fn read_id_from_code(pool: &mut DbPool<'_>, code_: &str) -> FastJobResult<LanguageId> {
    let conn = &mut get_conn(pool).await?;
    let res = language::table
      .filter(language::code.eq(code_))
      .first::<Self>(conn)
      .await
      .map(|l| l.id);

    // Return undetermined by default
    Ok(res.unwrap_or(UNDETERMINED_ID))
  }
}

#[cfg(test)]
mod tests {

  use crate::{source::language::Language, utils::build_db_pool_for_tests};
  use app_108jobs_core::error::FastJobResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_languages() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let all = Language::read_all(pool).await?;

    // This deployment seeds only four languages (see migration
    // 2022-06-21-123144_language-tags): und, en, th, vi. `read_all` has no
    // explicit ordering, so assert on membership rather than position.
    assert_eq!(4, all.len());
    let codes: Vec<&str> = all.iter().map(|l| l.code.as_str()).collect();
    assert!(codes.contains(&"und"), "missing und in {codes:?}");
    assert!(codes.contains(&"en"), "missing en in {codes:?}");
    assert!(codes.contains(&"th"), "missing th in {codes:?}");
    assert!(codes.contains(&"vi"), "missing vi in {codes:?}");

    Ok(())
  }
}
