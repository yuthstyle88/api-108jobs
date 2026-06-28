use app_108jobs_api_utils::plugins::plugin_metadata;
use app_108jobs_core::{
  error::{FastJobErrorExt, FastJobErrorType, FastJobResult},
  settings::structs::Settings,
  VERSION,
};
use app_108jobs_db::{
  schema::local_site,
  source::{
    actor_language::SiteLanguage,
    coin::CoinModel,
    instance::Instance,
    language::Language,
    local_site::{LocalSite, LocalSiteInsertForm},
    local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitInsertForm},
    local_site_url_blocklist::LocalSiteUrlBlocklist,
    local_user::{LocalUser, LocalUserInsertForm},
    oauth_provider::OAuthProvider,
    person::{Person, PersonInsertForm},
    site::{Site, SiteInsertForm},
    tagline::Tagline,
    wallet::WalletModel,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use app_108jobs_db_views_person::impls::PersonQuery;
use app_108jobs_db_views_site::{api::SiteSnapshot, SiteView};
use chrono::Utc;
use diesel::{
  dsl::{exists, not, select},
  query_builder::AsQuery,
};
use diesel_async::{scoped_futures::ScopedFutureExt, RunQueryDsl};
use tracing::info;

pub async fn setup_local_site(
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> FastJobResult<SiteSnapshot> {
  let conn = &mut get_conn(pool).await?;
  // Check to see if local_site exists, without the cache wrapper
  if select(not(exists(local_site::table.as_query())))
    .get_result(conn)
    .await?
  {
    info!("No Local Site found, creating it.");

    let domain = settings
      .get_hostname_without_port()
      .with_fastjob_type(FastJobErrorType::Unknown("must have domain".into()))?;
    conn
      .run_transaction(|conn| {
        async move {
          // Upsert this to the instance table
          let instance = Instance::read_or_create(&mut conn.into(), domain).await?;

          // Ensure platform assets exist (idempotent - seeded by migration)
          let platform_wallet = WalletModel::ensure_platform_wallet(&mut conn.into()).await?;
          let platform_coin = CoinModel::ensure_platform_coin(&mut conn.into()).await?;

          if let Some(setup) = &settings.setup {
            let public_key = Some("public_key".to_string());
            let private_key = Some("private_key".to_string());
            let wallet_id = Some(platform_wallet.id);
            // Register the user if there's a site setup
            let person_form = PersonInsertForm {
              private_key,
              wallet_id,
              ..PersonInsertForm::new(setup.admin_username.clone(), public_key, instance.id)
            };
            let person_inserted = Person::create(&mut conn.into(), &person_form).await?;

            let local_user_form = LocalUserInsertForm {
              email: setup.admin_email.clone(),
              admin: Some(true),
              ..LocalUserInsertForm::new(person_inserted.id, Some(setup.admin_password.clone()))
            };
            LocalUser::create(&mut conn.into(), &local_user_form, vec![]).await?;
          };

          // Add an entry for the site table
          let site_key_pair = "site_key_pair".to_string();

          let name = settings
            .setup
            .clone()
            .map(|s| s.site_name)
            .unwrap_or_else(|| "New Site".to_string());
          let site_form = SiteInsertForm {
            last_refreshed_at: Some(Utc::now()),
            private_key: Some(site_key_pair.clone()),
            public_key: Some(site_key_pair),
            ..SiteInsertForm::new(name, instance.id)
          };
          let site = Site::create(&mut conn.into(), &site_form).await?;

          // Finally create the local_site row with platform coin reference
          let local_site_form = LocalSiteInsertForm {
            site_setup: Some(settings.setup.is_some()),
            category_creation_admin_only: Some(true),
            coin_id: Some(platform_coin.id),
            ..LocalSiteInsertForm::new(site.id)
          };
          let local_site = LocalSite::create(&mut conn.into(), &local_site_form).await?;

          // Create the rate limit table
          let local_site_rate_limit_form = LocalSiteRateLimitInsertForm::new(local_site.id);
          LocalSiteRateLimit::create(&mut conn.into(), &local_site_rate_limit_form).await?;
          Ok(())
        }
        .scope_boxed()
      })
      .await?;
  }

  let site_view = SiteView::read_local(pool).await?;
  let admins = PersonQuery {
    admins_only: Some(true),
    ..Default::default()
  }
  .list(None, site_view.instance.id, pool)
  .await?;
  let all_languages = Language::read_all(pool).await?;
  let discussion_languages = SiteLanguage::read_local_raw(pool).await?;
  let blocked_urls = LocalSiteUrlBlocklist::get_all(pool).await?;
  let tagline = Tagline::get_random(pool).await.ok();
  let admin_oauth_providers = OAuthProvider::get_all(pool).await?;
  let oauth_providers = OAuthProvider::convert_providers_to_public(admin_oauth_providers.clone());

  Ok(SiteSnapshot {
    site_view: site_view.clone(),
    admins,
    version: VERSION.to_string(),
    all_languages,
    discussion_languages,
    blocked_urls,
    tagline,
    oauth_providers,
    admin_oauth_providers,
    image_upload_disabled: settings.pictrs()?.image_upload_disabled,
    active_plugins: plugin_metadata(),
  })
}
