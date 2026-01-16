use crate::{
  source::{
    instance::Instance,
    local_site::{LocalSite, LocalSiteInsertForm},
    local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitInsertForm},
    site::{Site, SiteInsertForm},
  },
  traits::Crud,
  utils::DbPool,
};
use app_108jobs_utils::error::FastJobResult;

pub struct TestData {
  pub instance: Instance,
  pub site: Site,
}

impl TestData {
  pub async fn create(pool: &mut DbPool<'_>) -> FastJobResult<Self> {
    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let site_form = SiteInsertForm::new("test site".to_string(), instance.id);
    let site = Site::create(pool, &site_form).await?;

    let local_site_form = LocalSiteInsertForm {
      ..LocalSiteInsertForm::new(site.id)
    };
    let local_site = LocalSite::create(pool, &local_site_form).await?;
    LocalSiteRateLimit::create(pool, &LocalSiteRateLimitInsertForm::new(local_site.id)).await?;

    Ok(Self { instance, site })
  }

  pub async fn delete(self, pool: &mut DbPool<'_>) -> FastJobResult<()> {
    Instance::delete(pool, self.instance.id).await?;
    Site::delete(pool, self.site.id).await?;
    Ok(())
  }
}
