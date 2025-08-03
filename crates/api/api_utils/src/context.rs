use lemmy_db_schema::{
  source::secret::Secret,
  utils::{ActualDbPool, DbPool},
};
use lemmy_utils::{
  rate_limit::RateLimit,
  settings::{structs::Settings, SETTINGS},
};
use reqwest_middleware::ClientWithMiddleware;
use std::sync::Arc;
use lemmy_utils::redis::RedisClient;

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
}

impl FastJobContext {
  pub fn create(
    pool: ActualDbPool,
    client: ClientWithMiddleware,
    pictrs_client: ClientWithMiddleware,
    secret: Secret,
    rate_limit_cell: RateLimit,
    redis: RedisClient,
  ) -> FastJobContext {
    FastJobContext {
      // Wrap all fields in Arc to reduce cloning overhead
      pool: Arc::new(pool),
      client: Arc::new(client),
      pictrs_client: Arc::new(pictrs_client),
      secret: Arc::new(secret),
      rate_limit_cell: Arc::new(rate_limit_cell),
      redis: Arc::new(redis),
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

}
