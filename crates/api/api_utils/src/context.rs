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
use p256::ecdh::EphemeralSecret;


#[derive(Clone)]
pub struct FastJobContext {
  pool: ActualDbPool,
  client: Arc<ClientWithMiddleware>,
  /// Pictrs requests must bypass proxy. Unfortunately no_proxy can only be set on ClientBuilder
  /// and not on RequestBuilder, so we need a separate client here.
  pictrs_client: Arc<ClientWithMiddleware>,
  secret: Arc<Secret>,
  rate_limit_cell: RateLimit,
  public_key: Vec<u8>,
  crypto_secret: Arc<EphemeralSecret>,
}

impl FastJobContext {
  pub fn create(
    pool: ActualDbPool,
    client: ClientWithMiddleware,
    pictrs_client: ClientWithMiddleware,
    secret: Secret,
    rate_limit_cell: RateLimit,
    public_key: Vec<u8>,
    crypto_secret: EphemeralSecret,
  ) -> FastJobContext {
    FastJobContext {
      pool,
      client: Arc::new(client),
      pictrs_client: Arc::new(pictrs_client),
      secret: Arc::new(secret),
      rate_limit_cell,
      public_key,
      crypto_secret: Arc::new(crypto_secret),
    }
  }
  pub fn pool(&self) -> DbPool<'_> {
    DbPool::Pool(&self.pool)
  }
  pub fn inner_pool(&self) -> &ActualDbPool {
    &self.pool
  }
  pub fn client(&self) -> &ClientWithMiddleware {
    &self.client
  }
  pub fn pictrs_client(&self) -> &ClientWithMiddleware {
    &self.pictrs_client
  }
  pub fn settings(&self) -> &'static Settings {
    &SETTINGS
  }
  pub fn secret(&self) -> &Secret {
    &self.secret
  }
  pub fn rate_limit_cell(&self) -> &RateLimit {
    &self.rate_limit_cell
  }
  pub fn public_key(&self) -> &[u8] {
    &self.public_key
  }
  pub fn crypto_secret(&self) -> &EphemeralSecret {
    &self.crypto_secret
  }


}
