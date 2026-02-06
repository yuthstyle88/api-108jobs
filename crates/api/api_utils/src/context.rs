use app_108jobs_db_schema::{
  source::secret::Secret,
  utils::{ActualDbPool, DbPool},
};
use app_108jobs_utils::error::FastJobResult;
use app_108jobs_utils::redis::RedisClient;
use app_108jobs_utils::{
  rate_limit::RateLimit,
  settings::{structs::Settings, SETTINGS},
};
use reqwest_middleware::ClientWithMiddleware;
use std::sync::Arc;
use crate::site_snapshot::SiteConfigProvider;

#[derive(Clone)]
pub struct FastJobContext {
  // Wrap pool in Arc to avoid expensive clones
  pool: Arc<ActualDbPool>,
  client: Arc<ClientWithMiddleware>,
  /// Pictrs requests must bypass proxy. Unfortunately no_proxy can only be set on ClientBuilder
  /// and not on RequestBuilder, so we need a separate client here.
  pictrs_client: Arc<ClientWithMiddleware>,
  secret: Arc<Secret>,
  // Wrap rate_limit_cell in Arc to avoid expensive clones
  rate_limit_cell: Arc<RateLimit>,
  redis: Arc<RedisClient>,
  site_config: Arc<dyn SiteConfigProvider + Send + Sync>,
  scb: Arc<ClientWithMiddleware>
}

impl FastJobContext {
  pub fn create(
    pool: ActualDbPool,
    client: ClientWithMiddleware,
    pictrs_client: ClientWithMiddleware,
    secret: Secret,
    rate_limit_cell: RateLimit,
    redis: RedisClient,
    site_config: Box<dyn SiteConfigProvider + Send + Sync>,
    scb: ClientWithMiddleware,
  ) -> FastJobContext {
    FastJobContext {
      // Wrap all fields in Arc to reduce cloning overhead
      pool: Arc::new(pool),
      client: Arc::new(client),
      pictrs_client: Arc::new(pictrs_client),
      secret: Arc::new(secret),
      rate_limit_cell: Arc::new(rate_limit_cell),
      redis: Arc::new(redis),
      site_config: Arc::from(site_config),
      scb: Arc::new(scb)
    }
  }
  
  // Update accessor methods to work with Arc-wrapped fields
  pub fn pool(&self) -> DbPool<'_> {
    // Create a DbPool that references the Arc-wrapped pool
    DbPool::Pool(&self.pool)
  }
  
  pub fn inner_pool(&self) -> &ActualDbPool {
    // Dereference the Arc to get the underlying pool
    &self.pool
  }
  
  pub fn client(&self) -> &ClientWithMiddleware {
    // Return a reference to the ClientWithMiddleware inside the Arc
    &self.client
  }
  
  pub fn pictrs_client(&self) -> &ClientWithMiddleware {
    // Return a reference to the pictrs_client inside the Arc
    &self.pictrs_client
  }
  
  pub fn settings(&self) -> &'static Settings {
    // This doesn't need to change as it's a static reference
    &SETTINGS
  }
  
  pub fn secret(&self) -> &Secret {
    // Return a reference to the Secret inside the Arc
    &self.secret
  }
  
  pub fn rate_limit_cell(&self) -> &RateLimit {
    // Return a reference to the RateLimit inside the Arc
    &self.rate_limit_cell
  }

  pub fn redis(&self) -> &RedisClient {
    // Return a reference to the RedisClient inside the Arc
    &self.redis
  }

  pub fn site_config(&self) -> &(dyn SiteConfigProvider + Send + Sync) {
    // Return a reference to the SiteView inside the Arc
    self.site_config.as_ref()
  }

  pub fn scb(&self) -> &ClientWithMiddleware {
    // Return a reference to the scb inside the Arc
    &self.scb
  }

  /// Get the coin_id from site configuration.
  /// Returns error if coin_id is not set.
  pub async fn get_coin_id(&self) -> FastJobResult<app_108jobs_db_schema::newtypes::CoinId> {
    let site_view = self.site_config().get().await?.site_view;
    site_view
      .local_site
      .coin_id
      .ok_or_else(|| anyhow::anyhow!("Coin ID not set").into())
  }

  /// Get the platform wallet ID from the first admin.
  /// Returns error if no admin is configured.
  pub async fn get_platform_wallet_id(&self) -> FastJobResult<app_108jobs_db_schema::newtypes::WalletId> {
    match self.site_config().get().await?.admins.first() {
      Some(a) => Ok(a.person.wallet_id),
      None => Err(app_108jobs_utils::error::FastJobErrorType::NoPlatformAdminConfigured.into()),
    }
  }

  #[allow(clippy::expect_used)]
  pub async fn init_test_federation_config() -> FastJobResult<()> {
    // call this to run migrations
    Ok(())
  }
  pub async fn init_test_context() -> FastJobResult<()> {
    let config = Self::init_test_federation_config().await;
    config
  }
}
