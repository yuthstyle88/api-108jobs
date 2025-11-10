use chrono::Utc;
use diesel::{
  dsl::{exists, not, select},
  query_builder::AsQuery,
};
use diesel_async::{scoped_futures::ScopedFutureExt, RunQueryDsl};
use lemmy_api_utils::plugins::plugin_metadata;
use lemmy_api_utils::utils::generate_inbox_url;
use lemmy_db_schema::source::actor_language::SiteLanguage;
use lemmy_db_schema::source::language::Language;
use lemmy_db_schema::source::local_site_url_blocklist::LocalSiteUrlBlocklist;
use lemmy_db_schema::source::oauth_provider::OAuthProvider;
use lemmy_db_schema::source::tagline::Tagline;
use lemmy_db_schema::source::wallet::WalletModel;
use lemmy_db_schema::source::coin::{CoinModel, CoinModelInsertForm};
use lemmy_db_schema::newtypes::Coin;
use lemmy_db_schema::{
  source::{
    instance::Instance,
    local_site::{LocalSite, LocalSiteInsertForm},
    local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitInsertForm},
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm},
    site::{Site, SiteInsertForm},
  },
  traits::{ApubActor, Crud},
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::local_site;
use lemmy_db_views_person::impls::PersonQuery;
use lemmy_db_views_site::api::SiteSnapshot;
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::{FastJobErrorExt, FastJobErrorType, FastJobResult},
  settings::structs::Settings,
  VERSION,
};
use tracing::info;
use url::Url;

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
     let supply_minted_total = settings.supply_minted_total;
    conn
      .run_transaction(|conn| {
        async move {
          // Upsert this to the instance table
          let instance = Instance::read_or_create(&mut conn.into(), domain).await?;

          if let Some(setup) = &settings.setup {
            let person_ap_id = Person::generate_local_actor_url(&setup.admin_username, settings)?;
            let public_key = Some("public_key".to_string());
            let private_key = Some("private_key".to_string());
            let wallet = WalletModel::create_for_platform(conn).await?;
            let wallet_id = Some(wallet.id);
            // Register the user if there's a site setup
            let person_form = PersonInsertForm {
              ap_id: Some(person_ap_id.clone()),
              inbox_url: Some(generate_inbox_url()?),
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
          let site_ap_id = Url::parse(&settings.get_protocol_and_hostname())?;

          let name = settings
            .setup
            .clone()
            .map(|s| s.site_name)
            .unwrap_or_else(|| "New Site".to_string());
          let site_form = SiteInsertForm {
            ap_id: Some(site_ap_id.clone().into()),
            last_refreshed_at: Some(Utc::now()),
            inbox_url: Some(generate_inbox_url()?),
            private_key: Some(site_key_pair.clone()),
            public_key: Some(site_key_pair),
            ..SiteInsertForm::new(name, instance.id)
          };
          let site = Site::create(&mut conn.into(), &site_form).await?;

          // Create default platform coin
          let total_supply = Coin(supply_minted_total);
          let coin_form = CoinModelInsertForm::new(
            "108JC".to_string(),
            "108Jobs Coin".to_string(),
            Some(total_supply),
            Some(total_supply), // Set minted equal to total initially
          );
          let coin = CoinModel::create(&mut conn.into(), &coin_form).await?;

          // Finally create the local_site row with coin reference
          let local_site_form = LocalSiteInsertForm {
            site_setup: Some(settings.setup.is_some()),
            category_creation_admin_only: Some(true),
            coin_id: Some(coin.id),
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
