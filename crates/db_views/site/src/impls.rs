use crate::api::{GetSiteResponse, Login, LoginRequest, SiteResponse, SiteSnapshot};
use crate::{api::UserSettingsBackup, SiteView};
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::sensitive::SensitiveString;
use lemmy_db_schema::utils::get_required_sensitive;
use lemmy_db_schema::{
    impls::local_user::UserBackupLists,
    utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::{instance, local_site, local_site_rate_limit, site};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::utils::validation::password_length_check;
use lemmy_utils::{build_cache, error::{FastJobError, FastJobErrorType, FastJobResult}, CacheLock, VERSION};
use std::sync::{Arc, LazyLock};

impl SiteView {
    pub async fn read_local(pool: &mut DbPool<'_>) -> FastJobResult<Self> {
        static CACHE: CacheLock<SiteView> = LazyLock::new(build_cache);
        CACHE
            .try_get_with((), async move {
                let conn = &mut get_conn(pool).await?;
                let local_site = site::table
                    .inner_join(local_site::table)
                    .inner_join(instance::table)
                    .inner_join(
                        local_site_rate_limit::table
                            .on(local_site::id.eq(local_site_rate_limit::local_site_id)),
                    )
                    .select(Self::as_select())
                    .first(conn)
                    .await
                    .optional()?
                    .ok_or(FastJobErrorType::LocalSiteNotSetup)?;
                Ok(local_site)
            })
            .await
            .map_err(|e: Arc<FastJobError>| anyhow::anyhow!("err getting local site: {e:?}").into())
    }
}

pub fn user_backup_list_to_user_settings_backup(
    local_user_view: LocalUserView,
    lists: UserBackupLists,
) -> UserSettingsBackup {
    let vec_into = |vec: Vec<_>| vec.into_iter().map(Into::into).collect();

    UserSettingsBackup {
        display_name: local_user_view.person.display_name,
        bio: local_user_view.person.bio,
        avatar: local_user_view.person.avatar.map(Into::into),
        banner: local_user_view.person.banner.map(Into::into),
        matrix_id: local_user_view.person.matrix_user_id,
        bot_account: local_user_view.person.bot_account.into(),
        settings: Some(local_user_view.local_user),
        blocked_users: vec_into(lists.blocked_users),
        saved_posts: vec_into(lists.saved_posts),
        saved_comments: vec_into(lists.saved_comments),
    }
}

impl TryFrom<LoginRequest> for Login {
    type Error = FastJobError;

    fn try_from(form: LoginRequest) -> Result<Self, Self::Error> {
        let username_or_email = get_required_sensitive(&form.username_or_email, FastJobErrorType::EmptyUsernameOrEmail)?;
        let password = get_required_sensitive(&form.password, FastJobErrorType::EmptyPassword)?;

        // Check password length (and other password policy if needed)
        password_length_check(&password)?;

        Ok(Self {
            username_or_email: SensitiveString::from(username_or_email.to_string()),
            password: SensitiveString::from(password.to_string()),
            totp_2fa_token: form.totp_2fa_token,
        })
    }
}

impl From<SiteSnapshot> for GetSiteResponse {
    fn from(v: SiteSnapshot) -> Self {
        Self {
            site_view: v.site_view,
            admins: v.admins,
            version: v.version,
            all_languages: v.all_languages,
            discussion_languages: v.discussion_languages,
            blocked_urls: v.blocked_urls,
            tagline: v.tagline,
            oauth_providers: v.oauth_providers,
            admin_oauth_providers: v.admin_oauth_providers,
            image_upload_disabled: v.image_upload_disabled,
            active_plugins: v.active_plugins,
        }
    }
}
