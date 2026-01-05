use crate::{
  source::local_site_url_blocklist::{LocalSiteUrlBlocklist, LocalSiteUrlBlocklistForm},
  utils::{get_conn, DbPool},
};
use diesel::dsl::insert_into;
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncPgConnection, RunQueryDsl};
use app_108jobs_db_schema_file::schema::local_site_url_blocklist;
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl LocalSiteUrlBlocklist {
  pub async fn replace(pool: &mut DbPool<'_>, url_blocklist: Vec<String>) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;

    conn
      .run_transaction(|conn| {
        async move {
          Self::clear(conn).await?;

          let forms = url_blocklist
            .into_iter()
            .map(|url| LocalSiteUrlBlocklistForm {
              url,
              updated_at: None,
            })
            .collect::<Vec<_>>();

          insert_into(local_site_url_blocklist::table)
            .values(forms)
            .execute(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntUpdateLocalSiteUrlBlocklist)
        }
        .scope_boxed()
      })
      .await
  }

  async fn clear(conn: &mut AsyncPgConnection) -> FastJobResult<usize> {
    diesel::delete(local_site_url_blocklist::table)
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::Deleted)
  }

  pub async fn get_all(pool: &mut DbPool<'_>) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    local_site_url_blocklist::table
      .get_results::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}
