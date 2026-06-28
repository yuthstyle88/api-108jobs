use crate::{
  newtypes::{InstanceId, SiteId},
  schema::{local_site, site},
  source::{
    actor_language::SiteLanguage,
    site::{Site, SiteInsertForm, SiteUpdateForm},
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use app_108jobs_core::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use diesel::{dsl::insert_into, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

impl Crud for Site {
  type InsertForm = SiteInsertForm;
  type UpdateForm = SiteUpdateForm;
  type IdType = SiteId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    let site = insert_into(site::table)
      .values(form)
      .get_result::<Self>(conn)
      .await?;

    // initialize languages for new site
    SiteLanguage::update(pool, vec![], &site).await?;

    Ok(site)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    site_id: SiteId,
    new_site: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(site::table.find(site_id))
      .set(new_site)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateSite)
  }
}

impl Site {
  pub async fn read_from_instance_id(
    pool: &mut DbPool<'_>,
    instance_id: InstanceId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    site::table
      .filter(site::instance_id.eq(instance_id))
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
  pub async fn read_local(pool: &mut DbPool<'_>) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    site::table
      .inner_join(local_site::table)
      .select(site::all_columns)
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::LocalSiteNotSetup)
  }
}
