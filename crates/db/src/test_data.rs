//! Shared test fixture.
//!
//! Many `#[cfg(test)]` modules across this crate (e.g. `impls::actor_language`,
//! `impls::local_site`) expect a minimal site + instance scaffold to be in
//! place before they exercise their own behavior. This module provides that
//! scaffold via [`TestData`].
//!
//! Surface:
//!   * [`TestData::create`] creates a fresh `Instance`, a `Site` row that references it, and a
//!     `LocalSite` row that references the `Site`.
//!   * `data.instance` and `data.site` are exposed as plain public fields so the existing test code
//!     can use `data.instance.id` / `&data.site` as it does today.
//!   * [`TestData::delete`] deletes the `Instance`; the cascading FKs on `site(instance_id)` and
//!     `local_site(site_id)` clean up the rest.
//!
//! The module is gated `#[cfg(test)]` because nothing in production should
//! ever depend on it.

use crate::{
  source::{
    instance::Instance,
    local_site::{LocalSite, LocalSiteInsertForm},
    site::{Site, SiteInsertForm},
  },
  traits::Crud,
  utils::DbPool,
};
use app_108jobs_core::error::FastJobResult;
use std::sync::atomic::{AtomicU64, Ordering};

/// Counter used to generate unique domains within a single test process. We
/// also mix in `std::process::id()` so parallel cargo-test invocations across
/// processes don't collide on the same row.
static TEST_DATA_SEQ: AtomicU64 = AtomicU64::new(0);

/// Build a UUID-suffixed domain so each test that calls
/// `Instance::read_or_create(pool, unique_test_domain("foo"))` gets its own
/// instance row. Prevents cross-test pollution on the shared `my_domain.tld`
/// instance: e.g. `category_pkey` collisions from prior categories, or the
/// `test_email_taken` assertion flipping on a re-run that found leftover
/// rows.
///
/// `prefix` is embedded so postmortem inspection of leftover rows points at
/// the originating test.
pub fn unique_test_domain(prefix: &str) -> String {
  format!("{prefix}-{}.example.test", uuid::Uuid::new_v4().simple())
}

/// Resync the `category_id_seq` past the highest existing `category.id`.
///
/// Migration `2025-09-25-060356-0000_init_categories` inserts categories with
/// explicit ids but never calls `setval()`, so a freshly-built test DB has
/// `category_id_seq` returning 1 while ids 1..N are already taken by the seed.
/// Every test that inserts a category via SERIAL collides on `category_pkey`.
///
/// This helper runs the resync as a no-op-safe `setval`. Pure test-fixture
/// fix — the schema and migrations are NOT modified. Call it once at the
/// start of any test that creates a category.
pub async fn reset_category_sequence(pool: &mut DbPool<'_>) -> FastJobResult<()> {
  use diesel::sql_query;
  use diesel_async::RunQueryDsl;
  let conn = &mut crate::utils::get_conn(pool).await?;
  // GREATEST(..., 1) keeps setval valid even on an empty table.
  sql_query(
    "SELECT setval('category_id_seq', \
         GREATEST(COALESCE((SELECT MAX(id) FROM category), 0), 1))",
  )
  .execute(&mut *conn)
  .await
  .map_err(app_108jobs_core::error::FastJobError::from)?;
  Ok(())
}

pub struct TestData {
  pub instance: Instance,
  pub site: Site,
}

/// Idempotently point `SETTINGS` at the workspace-root `config/config.hjson`
/// so tests work whether `cargo test` runs from the crate dir or the
/// workspace root.
///
/// Must be called BEFORE `build_db_pool_for_tests()` (which forces the
/// `SETTINGS` LazyLock to materialize). Safe to call multiple times — env
/// mutation is idempotent with the same value, and the LazyLock only reads
/// it once.
pub fn init_test_settings_path() {
  // CARGO_MANIFEST_DIR is `crates/db_schema` here; `../../config/config.hjson`
  // resolves to the workspace-root config. The same expression works for any
  // downstream crate that also sits one level deep under `crates/`.
  if std::env::var("app_108jobs_CONFIG_LOCATION").is_err() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_config = format!("{manifest_dir}/../../config/config.hjson");
    // SAFETY: tests run single-threaded under #[serial], and we're idempotent.
    std::env::set_var("app_108jobs_CONFIG_LOCATION", workspace_config);
  }
}

/// One-shot helper: configure the settings path and return a fresh
/// `ActualDbPool`. Equivalent to calling [`init_test_settings_path`] +
/// `build_db_pool_for_tests` and used by all DB-backed tests so each one
/// is robust to cargo's CWD.
pub fn pool_for_tests() -> crate::utils::ActualDbPool {
  init_test_settings_path();
  crate::utils::build_db_pool_for_tests()
}

impl TestData {
  /// Create a fresh instance + site + local_site triple. Returns owned values
  /// so the caller can chain `data.instance.id` / `&data.site` as in the
  /// existing tests.
  pub async fn create(pool: &mut DbPool<'_>) -> FastJobResult<Self> {
    let seq = TEST_DATA_SEQ.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let domain = format!("test-data-{pid}-{seq}.tld");

    let instance = Instance::read_or_create(pool, domain.clone()).await?;

    let form = SiteInsertForm {
      name: format!("test site {seq}"),
      instance_id: instance.id,
      sidebar: None,
      updated_at: None,
      icon: None,
      banner: None,
      description: None,
      last_refreshed_at: None,
      content_warning: None,
    };
    let site = Site::create(pool, &form).await?;

    // local_site is a process-wide singleton in this schema (idempotently
    // unique on site_id). If a previous test forgot to clean it up, surface
    // that as a hard failure rather than masking it — but we still try to
    // create one so the common case "fresh DB" works.
    let local_form = LocalSiteInsertForm::new(site.id);
    let _ = LocalSite::create(pool, &local_form).await?;

    Ok(Self { instance, site })
  }

  /// Tear down everything created by [`create`]. Cascading FKs on
  /// `site(instance_id)` and `local_site(site_id)` mean a single
  /// `Instance::delete` removes the full chain.
  ///
  /// Consumes `self` to discourage post-delete use.
  pub async fn delete(self, pool: &mut DbPool<'_>) -> FastJobResult<()> {
    // Best-effort: if local_site was already deleted by the test (the
    // existing actor_language tests do this), we still want Instance::delete
    // to succeed. We ignore the result of an explicit LocalSite::delete here
    // because the cascade will pick up whatever remains.
    Instance::delete(pool, self.instance.id).await?;
    Ok(())
  }
}
