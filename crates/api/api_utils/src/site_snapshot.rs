use lemmy_db_schema::source::actor_language::SiteLanguage;
use lemmy_db_schema::source::language::Language;
use lemmy_db_schema::source::local_site_url_blocklist::LocalSiteUrlBlocklist;
use lemmy_db_schema::source::oauth_provider::OAuthProvider;
use lemmy_db_schema::source::tagline::Tagline;
use lemmy_db_schema::utils::{ActualDbPool, DbPool};
use lemmy_db_views_person::impls::PersonQuery;
use lemmy_db_views_site::api::SiteSnapshot;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::FastJobResult;
use lemmy_utils::settings::structs::Settings;
use lemmy_utils::VERSION;
use std::sync::{Arc, RwLock};
use crate::plugins::plugin_metadata;
use futures::future::BoxFuture;
use futures::FutureExt;

pub trait SiteConfigProvider: Send + Sync {
    /// Returns a cached snapshot; impl is responsible for warming/refreshing.
    fn get<'a>(&'a self) -> BoxFuture<'a, FastJobResult<SiteSnapshot>>;
    /// Optional: force refresh after updates in local_site.
    fn refresh<'a>(&'a self) -> BoxFuture<'a, FastJobResult<()>>;
}

pub struct CachedSiteConfigProvider {
    cache: RwLock<SiteSnapshot>,
    pool: ActualDbPool,
    settings: Settings
}

impl CachedSiteConfigProvider {
    pub fn new(pool: ActualDbPool, initial_snapshot: SiteSnapshot, settings: Settings) -> Self {
        Self {
            cache: RwLock::new(initial_snapshot),
            pool,
            settings
        }
    }

    async fn fetch_from_source(&self) -> FastJobResult<SiteSnapshot> {
        let mut pool = DbPool::Pool(&self.pool);

        let site_view = SiteView::read_local(&mut pool).await?;
        let admins = PersonQuery {
            admins_only: Some(true),
            ..Default::default()
        }
            .list(None, site_view.instance.id, &mut pool)
            .await?;
        let all_languages = Language::read_all(&mut pool).await?;
        let discussion_languages = SiteLanguage::read_local_raw(&mut pool).await?;
        let blocked_urls = LocalSiteUrlBlocklist::get_all(&mut pool).await?;
        let tagline = Tagline::get_random(&mut pool).await.ok();
        let admin_oauth_providers = OAuthProvider::get_all(&mut pool).await?;
        let oauth_providers = OAuthProvider::convert_providers_to_public(admin_oauth_providers.clone());

        Ok(SiteSnapshot {
            site_view: site_view.clone(),
            admins,
            version: VERSION.to_string(),
            all_languages,
            discussion_languages,
            blocked_urls,
            image_upload_disabled: self.settings.pictrs()?.image_upload_disabled,
            tagline,
            oauth_providers,
            admin_oauth_providers,
            active_plugins: plugin_metadata(),
        })
    }

    pub fn start_background_refresh(self: Arc<Self>, interval_secs: u64) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));
            loop {
                interval.tick().await;
                if let Err(e) = self.refresh().await {
                    eprintln!("Failed to refresh site config: {}", e);
                }
            }
        });
    }
}

impl SiteConfigProvider for CachedSiteConfigProvider {
    fn get<'a>(&'a self) -> BoxFuture<'a, FastJobResult<SiteSnapshot>> {
        async move {
            let snapshot = self
                .cache
                .read()
                .map_err(|e| anyhow::anyhow!("RwLock poisoned: {}", e))?;
            Ok(snapshot.clone())
        }
        .boxed()
    }

    fn refresh<'a>(&'a self) -> BoxFuture<'a, FastJobResult<()>> {
        async move {
            let new_snapshot = self.fetch_from_source().await?;
            let mut cache = self
                .cache
                .write()
                .map_err(|e| anyhow::anyhow!("RwLock poisoned: {}", e))?;
            *cache = new_snapshot;
            Ok(())
        }
        .boxed()
    }
}