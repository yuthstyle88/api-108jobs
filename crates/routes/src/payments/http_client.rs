//! Shared HTTP client for SCB outbound calls. Centralises timeouts so a
//! single `Client::new()` regression cannot leak in.
//!
//! Connect / request / total timeouts are tuned for a synchronous merchant
//! API. If SCB is slow, fail fast and surface the error to the caller rather
//! than starving the actix worker pool.

use reqwest::Client;
use std::time::Duration;

/// Hard cap on a single SCB request. The merchant API is interactive, so a
/// real call should complete in well under a second. 15s is a generous upper
/// bound that still lets us fail fast under outage conditions.
pub const SCB_REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// TCP connect deadline. SCB is one host — we don't need to wait minutes to
/// notice it's unreachable.
pub const SCB_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Build a single-use client with all timeouts wired in. Cheap on the hot
/// path; reqwest reuses the underlying connection pool internally.
pub fn scb_client() -> Result<Client, reqwest::Error> {
  Client::builder()
    .timeout(SCB_REQUEST_TIMEOUT)
    .connect_timeout(SCB_CONNECT_TIMEOUT)
    .pool_idle_timeout(Duration::from_secs(60))
    .build()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn client_builds_with_finite_timeouts() {
    let _ = scb_client().expect("client should build");
    // The Client API does not expose timeout introspection, but a successful
    // build means our timeout values are valid Duration values. Regression
    // guards live at the call sites that depend on these constants.
    assert!(SCB_REQUEST_TIMEOUT > Duration::from_secs(0));
    assert!(SCB_REQUEST_TIMEOUT <= Duration::from_secs(30));
    assert!(SCB_CONNECT_TIMEOUT < SCB_REQUEST_TIMEOUT);
  }
}
