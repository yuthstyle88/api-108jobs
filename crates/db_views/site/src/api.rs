use crate::SiteView;
use chrono::{DateTime, Utc};
use lemmy_db_schema::{
  newtypes::{InstanceId, LanguageId, OAuthProviderId, PaginationCursor, TaglineId},
  sensitive::SensitiveString,
  source::{
    comment::Comment,
    community::Community,
    instance::Instance,
    language::Language,
    local_site_url_blocklist::LocalSiteUrlBlocklist,
    local_user::LocalUser,
    login_token::LoginToken,
    oauth_provider::{OAuthProvider, PublicOAuthProvider},
    person::Person,
    post::Post,
    tagline::Tagline,
    wallet::Wallet,
  },
};
use lemmy_db_schema_file::enums::{
  CommentSortType, ListingType, PostListingMode, PostSortType, RegistrationMode, VoteShow,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::PersonView;
use lemmy_db_views_post::PostView;
use lemmy_utils::error::FastJobError;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;
use validator::Validate;
#[cfg(feature = "full")]
use {
  extism::FromBytes,
  extism_convert::{encoding, Json},
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminAllowInstanceParams {
  pub instance: String,
  pub allow: bool,
  pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct AdminBlockInstanceParams {
  pub instance: String,
  pub block: bool,
  pub reason: Option<String>,
  pub expires_at: Option<DateTime<Utc>>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Logging in with an OAuth 2.0 authorization
#[serde(rename_all = "camelCase")]
pub struct AuthenticateWithOauth {
  pub code: String,
  pub oauth_provider_id: OAuthProviderId,
  pub redirect_uri: Url,
  pub self_promotion: Option<bool>,
  /// Username is mandatory at registration time
  pub username: Option<String>,
  pub name: Option<String>,
  /// An answer is mandatory if require application is enabled on the server
  pub answer: Option<String>,
  pub pkce_code_verifier: Option<String>,
}
#[skip_serializing_none]
#[derive(Debug, Validate, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateWithOauthRequest {
  pub code: String,
  pub oauth_provider_id: OAuthProviderId,
  pub redirect_uri: Url,
  pub name: Option<String>,
  pub email: Option<String>,
  pub answer: Option<String>,
}
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct RegisterWithOauthRequest {
  pub code: String,
  pub oauth_provider_id: OAuthProviderId,
  pub redirect_uri: Url,
  pub name: Option<String>,
  pub email: String,
  pub self_promotion: Option<bool>,
  pub answer: Option<String>,
  pub pkce_code_verifier: Option<String>,
}
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct EmailExistsRequest {
  pub email: String,
}

impl TryFrom<AuthenticateWithOauthRequest> for AuthenticateWithOauth {
  type Error = FastJobError;
  fn try_from(value: AuthenticateWithOauthRequest) -> Result<Self, Self::Error> {
    Ok(AuthenticateWithOauth {
      code: value.code,
      oauth_provider_id: value.oauth_provider_id,
      redirect_uri: value.redirect_uri,
      self_promotion: Some(false),
      username: Option::from(value.email),
      name: value.name,
      answer: value.answer,
      pkce_code_verifier: None,
    })
  }
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create an external auth method.
#[serde(rename_all = "camelCase")]
pub struct CreateOAuthProvider {
  pub display_name: String,
  pub issuer: String,
  pub authorization_endpoint: String,
  pub token_endpoint: String,
  pub userinfo_endpoint: String,
  pub id_claim: String,
  pub client_id: String,
  pub client_secret: String,
  pub scopes: String,
  pub auto_verify_email: Option<bool>,
  pub account_linking_enabled: Option<bool>,
  pub use_pkce: Option<bool>,
  pub enabled: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Creates a site. Should be done after first running lemmy.
#[serde(rename_all = "camelCase")]
pub struct CreateSite {
  pub name: String,
  pub sidebar: Option<String>,
  pub description: Option<String>,
  pub community_creation_admin_only: Option<bool>,
  pub require_email_verification: Option<bool>,
  pub application_question: Option<String>,
  pub private_instance: Option<bool>,
  pub default_theme: Option<String>,
  pub default_post_listing_type: Option<ListingType>,
  pub default_post_listing_mode: Option<PostListingMode>,
  pub default_post_sort_type: Option<PostSortType>,
  pub default_post_time_range_seconds: Option<i32>,
  pub default_comment_sort_type: Option<CommentSortType>,
  pub legal_information: Option<String>,
  pub application_email_admins: Option<bool>,
  pub discussion_languages: Option<Vec<LanguageId>>,
  pub slur_filter_regex: Option<String>,
  pub actor_name_max_length: Option<i32>,
  pub rate_limit_message_max_requests: Option<i32>,
  pub rate_limit_message_interval_seconds: Option<i32>,
  pub rate_limit_post_max_requests: Option<i32>,
  pub rate_limit_post_interval_seconds: Option<i32>,
  pub rate_limit_register_max_requests: Option<i32>,
  pub rate_limit_register_interval_seconds: Option<i32>,
  pub rate_limit_image_max_requests: Option<i32>,
  pub rate_limit_image_interval_seconds: Option<i32>,
  pub rate_limit_comment_max_requests: Option<i32>,
  pub rate_limit_comment_interval_seconds: Option<i32>,
  pub rate_limit_search_max_requests: Option<i32>,
  pub rate_limit_search_interval_seconds: Option<i32>,
  pub rate_limit_import_user_settings_max_requests: Option<i32>,
  pub rate_limit_import_user_settings_interval_seconds: Option<i32>,
  pub captcha_enabled: Option<bool>,
  pub captcha_difficulty: Option<String>,
  pub registration_mode: Option<RegistrationMode>,
  pub oauth_registration: Option<bool>,
  pub content_warning: Option<String>,
  pub disallow_self_promotion_content: Option<bool>,
  pub disable_email_notifications: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Delete an external auth method.
pub struct DeleteOAuthProvider {
  pub id: OAuthProviderId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Edit an external auth method.
#[serde(rename_all = "camelCase")]
pub struct EditOAuthProvider {
  pub id: OAuthProviderId,
  pub display_name: Option<String>,
  pub authorization_endpoint: Option<String>,
  pub token_endpoint: Option<String>,
  pub userinfo_endpoint: Option<String>,
  pub id_claim: Option<String>,
  pub client_secret: Option<String>,
  pub scopes: Option<String>,
  pub auto_verify_email: Option<bool>,
  pub account_linking_enabled: Option<bool>,
  pub use_pkce: Option<bool>,
  pub enabled: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Edits a site.
#[serde(rename_all = "camelCase")]
pub struct EditSite {
  pub name: Option<String>,
  /// A sidebar for the site, in markdown.
  pub sidebar: Option<String>,
  /// A shorter, one line description of your site.
  pub description: Option<String>,
  /// Limits community creation to admins only.
  pub community_creation_admin_only: Option<bool>,
  /// Whether to require multilang verification.
  pub require_email_verification: Option<bool>,
  /// Your application question form. This is in markdown, and can be many questions.
  pub application_question: Option<String>,
  /// Whether your instance is public, or private.
  pub private_instance: Option<bool>,
  /// The default theme. Usually "browser"
  pub default_theme: Option<String>,
  /// The default post listing type, usually "local"
  pub default_post_listing_type: Option<ListingType>,
  /// Default value for listing mode, usually "list"
  pub default_post_listing_mode: Option<PostListingMode>,
  /// The default post sort, usually "active"
  pub default_post_sort_type: Option<PostSortType>,
  /// A default time range limit to apply to post sorts, in seconds. 0 means none.
  pub default_post_time_range_seconds: Option<i32>,
  /// The default comment sort, usually "hot"
  pub default_comment_sort_type: Option<CommentSortType>,
  /// An optional page of legal information
  pub legal_information: Option<String>,
  /// Whether to multilang admins when receiving a new application.
  pub application_email_admins: Option<bool>,
  /// A list of allowed discussion languages.
  pub discussion_languages: Option<Vec<LanguageId>>,
  /// A regex string of items to filter.
  pub slur_filter_regex: Option<String>,
  /// The max length of actor names.
  pub actor_name_max_length: Option<i32>,
  /// The number of messages allowed in a given time frame.
  pub rate_limit_message_max_requests: Option<i32>,
  pub rate_limit_message_interval_seconds: Option<i32>,
  /// The number of posts allowed in a given time frame.
  pub rate_limit_post_max_requests: Option<i32>,
  pub rate_limit_post_interval_seconds: Option<i32>,
  /// The number of registrations allowed in a given time frame.
  pub rate_limit_register_max_requests: Option<i32>,
  pub rate_limit_register_interval_seconds: Option<i32>,
  /// The number of image uploads allowed in a given time frame.
  pub rate_limit_image_max_requests: Option<i32>,
  pub rate_limit_image_interval_seconds: Option<i32>,
  /// The number of comments allowed in a given time frame.
  pub rate_limit_comment_max_requests: Option<i32>,
  pub rate_limit_comment_interval_seconds: Option<i32>,
  /// The number of searches allowed in a given time frame.
  pub rate_limit_search_max_requests: Option<i32>,
  pub rate_limit_search_interval_seconds: Option<i32>,
  /// The number of settings imports or exports allowed in a given time frame.
  pub rate_limit_import_user_settings_max_requests: Option<i32>,
  pub rate_limit_import_user_settings_interval_seconds: Option<i32>,
  /// Whether to enable captchas for signups.
  pub captcha_enabled: Option<bool>,
  /// The captcha difficulty. Can be easy, medium, or hard
  pub captcha_difficulty: Option<String>,
  /// A list of blocked URLs
  pub blocked_urls: Option<Vec<String>>,
  pub registration_mode: Option<RegistrationMode>,
  /// Whether to multilang admins for new reports.
  pub reports_email_admins: Option<bool>,
  /// If present, self_promotion content is visible by default. Should be displayed by frontends/clients
  /// when the site is first opened by a user.
  pub content_warning: Option<String>,
  /// Whether or not external auth methods can auto-register users.
  pub oauth_registration: Option<bool>,
  /// What kind of post upvotes your site allows.
  /// Block NSFW content being created
  pub disallow_self_promotion_content: Option<bool>,
  /// Dont send multilang notifications to users for new replies, mentions etc
  pub disable_email_notifications: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// An expanded response for a site.
#[serde(rename_all = "camelCase")]
pub struct GetSiteResponse {
  pub site_view: SiteView,
  pub admins: Vec<PersonView>,
  pub version: String,
  pub all_languages: Vec<Language>,
  pub discussion_languages: Vec<LanguageId>,
  /// If the site has any taglines, a random one is included here for displaying
  pub tagline: Option<Tagline>,
  /// A list of external auth methods your site supports.
  pub oauth_providers: Vec<PublicOAuthProvider>,
  pub admin_oauth_providers: Vec<OAuthProvider>,
  pub blocked_urls: Vec<LocalSiteUrlBlocklist>,
  // If true then uploads for post images or markdown images are disabled. Only avatars, icons and
  // banners can be set.
  pub image_upload_disabled: bool,
  pub active_plugins: Vec<PluginMetadata>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The response for a site.
#[serde(rename_all = "camelCase")]
pub struct SiteResponse {
  pub site_view: SiteView,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A captcha response.
#[serde(rename_all = "camelCase")]
pub struct CaptchaResponse {
  /// A Base64 encoded png
  pub png: String,
  /// A Base64 encoded wav audio
  pub wav: String,
  /// The UUID for the captcha item.
  pub uuid: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Changes your account password.
#[serde(rename_all = "camelCase")]
pub struct ChangePassword {
  pub new_password: SensitiveString,
  pub new_password_verify: SensitiveString,
  pub old_password: SensitiveString,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Delete your account.
#[serde(rename_all = "camelCase")]
pub struct DeleteAccount {
  pub password: SensitiveString,
  pub delete_content: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A wrapper for the captcha response.
pub struct GetCaptchaResponse {
  /// Will be None if captchas are disabled.
  pub ok: Option<CaptchaResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct GenerateTotpSecretResponse {
  pub totp_secret_url: SensitiveString,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ListLoginsResponse {
  pub logins: Vec<LoginToken>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Logging into lemmy.
///
/// Note: Banned users can still log in, to be able to do certain things like delete
/// their account.
#[serde(rename_all = "camelCase")]
pub struct Login {
  pub username_or_email: SensitiveString,
  pub password: SensitiveString,
  /// May be required, if totp is enabled for their account.
  pub totp_2fa_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
  pub username_or_email: Option<SensitiveString>,
  pub password: Option<SensitiveString>,
  pub totp_2fa_token: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct ExchangeKey {
  pub public_key: String,
}
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeKeyResponse {
  pub public_key: SensitiveString,
}
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[derive(Default)]
pub struct EmailExistsResponse {
  pub exists: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
  /// This is None in response to `Register` if multilang verification is enabled, or the server
  /// requires registration applications.
  pub jwt: Option<SensitiveString>,
  /// If registration applications are required, this will return true for a signup response.
  pub registration_created: bool,
  /// If multilang verifications are required, this will return true for a signup response.
  pub verify_email_sent: bool,
  pub application_pending: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Your user info.
#[serde(rename_all = "camelCase")]
pub struct MyUserInfo {
  pub local_user_view: LocalUserView,
  pub community_blocks: Vec<Community>,
  pub instance_blocks: Vec<Instance>,
  pub person_blocks: Vec<Person>,
  pub keyword_blocks: Vec<String>,
  pub discussion_languages: Vec<LanguageId>,
  pub wallet: Option<Wallet>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Change your password after receiving a reset request.
#[serde(rename_all = "camelCase")]
pub struct PasswordChangeAfterReset {
  pub token: SensitiveString,
  pub password: SensitiveString,
  pub password_verify: SensitiveString,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Reset your password via multilang.
pub struct PasswordReset {
  pub email: SensitiveString,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Make a request to resend your verification multilang.
pub struct ResendVerificationEmail {
  pub email: SensitiveString,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Saves settings for your user.
#[serde(rename_all = "camelCase")]
pub struct SaveUserSettings {
  /// Show self_promotion posts.
  pub self_promotion: Option<bool>,
  /// Blur self_promotion posts.
  pub blur_self_promotion: Option<bool>,
  /// Your user's theme.
  pub theme: Option<String>,
  /// The default post listing type, usually "local"
  pub default_listing_type: Option<ListingType>,
  /// A post-view mode that changes how multiple post listings look.
  pub post_listing_mode: Option<PostListingMode>,
  /// The default post sort, usually "active"
  pub default_post_sort_type: Option<PostSortType>,
  /// A default time range limit to apply to post sorts, in seconds. 0 means none.
  pub default_post_time_range_seconds: Option<i32>,
  /// The default comment sort, usually "hot"
  pub default_comment_sort_type: Option<CommentSortType>,
  /// The language of the lemmy interface
  pub interface_language: Option<String>,
  /// Your display name, which can contain strange characters, and does not need to be unique.
  pub display_name: Option<String>,
  /// Your multilang.
  pub email: Option<SensitiveString>,
  /// Your bio / info, in markdown.
  pub bio: Option<String>,
  /// Your matrix user id. Ex: @my_user:matrix.org
  pub matrix_user_id: Option<String>,
  /// Whether to show or hide avatars.
  pub show_avatars: Option<bool>,
  /// Sends notifications to your multilang.
  pub send_notifications_to_email: Option<bool>,
  /// Whether this account is a bot account. Users can hide these accounts easily if they wish.
  pub bot_account: Option<bool>,
  /// Whether to show bot accounts.
  pub show_bot_accounts: Option<bool>,
  /// Whether to show read posts.
  pub show_read_posts: Option<bool>,
  /// A list of languages you are able to see discussion in.
  pub discussion_languages: Option<Vec<LanguageId>>,
  // A list of keywords used for blocking posts having them in title,url or body.
  pub blocking_keywords: Option<Vec<String>>,
  /// Open links in a new tab
  pub open_links_in_new_tab: Option<bool>,
  /// Enable infinite scroll
  pub infinite_scroll_enabled: Option<bool>,
  /// Whether to allow keyboard navigation (for browsing and interacting with posts and comments).
  pub enable_keyboard_navigation: Option<bool>,
  /// Whether user avatars or inline images in the UI that are gifs should be allowed to play or
  /// should be paused
  pub enable_animated_images: Option<bool>,
  /// Whether to auto-collapse bot comments.
  pub collapse_bot_comments: Option<bool>,
  /// Some vote display mode settings
  pub show_scores: Option<bool>,
  pub show_upvotes: Option<bool>,
  pub show_downvotes: Option<VoteShow>,
  pub show_upvote_percentage: Option<bool>,
  /// Whether to automatically mark fetched posts as read.
  pub auto_mark_fetched_posts_as_read: Option<bool>,
  /// Whether to hide posts containing images/videos.
  pub hide_media: Option<bool>,
  /// Whether to show vote totals given to others.
  pub show_person_votes: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct UpdateTotp {
  pub totp_token: String,
  pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct UpdateTotpResponse {
  pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Block an instance as user
#[serde(rename_all = "camelCase")]
pub struct UserBlockInstanceParams {
  pub instance_id: InstanceId,
  pub block: bool,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Verify your VerifyEmail.
pub struct VerifyEmail {
  pub code: String,
}
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Verify your VerifyOTP.
pub struct VerifyOTP {
  pub otp: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyEmailSuccessResponse {
  pub jwt: SensitiveString,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets your hidden posts.
#[serde(rename_all = "camelCase")]
pub struct ListPersonHidden {
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// You hidden posts response.
#[serde(rename_all = "camelCase")]
pub struct ListPersonHiddenResponse {
  pub hidden: Vec<PostView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets your read posts.
#[serde(rename_all = "camelCase")]
pub struct ListPersonRead {
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// You read posts response.
#[serde(rename_all = "camelCase")]
pub struct ListPersonReadResponse {
  pub read: Vec<PostView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a tagline
pub struct CreateTagline {
  pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Delete a tagline
pub struct DeleteTagline {
  pub id: TaglineId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Fetches a list of taglines.
#[serde(rename_all = "camelCase")]
pub struct ListTaglines {
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A response for taglines.
#[serde(rename_all = "camelCase")]
pub struct ListTaglinesResponse {
  pub taglines: Vec<Tagline>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct TaglineResponse {
  pub tagline: Tagline,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Update a tagline
pub struct UpdateTagline {
  pub id: TaglineId,
  pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(FromBytes))]
#[cfg_attr(feature = "full", encoding(Json))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct PluginMetadata {
  name: String,
  url: Url,
  description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Does an apub fetch for an object.
pub struct ResolveObject {
  /// Can be the full url, or a shortened version like: !fediverse@lemmy.ml
  pub q: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub enum PostOrCommentOrPrivateMessage {
  Post(Post),
  Comment(Comment),
}

/// Backup of user data. This struct should never be changed so that the data can be used as a
/// long-term backup in case the instance goes down unexpectedly. All fields are optional to allow
/// importing partial backups.
///
/// This data should not be parsed by apps/clients, but directly downloaded as a file.
///
/// Be careful with any changes to this struct, to avoid breaking changes which could prevent
/// importing older backups.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct UserSettingsBackup {
  pub display_name: Option<String>,
  pub bio: Option<String>,
  pub avatar: Option<Url>,
  pub banner: Option<Url>,
  pub matrix_id: Option<String>,
  pub bot_account: Option<bool>,
  // TODO: might be worth making a separate struct for settings backup, to avoid breakage in case
  //       fields are renamed, and to avoid storing unnecessary fields like person_id or multilang
  pub settings: Option<LocalUser>,
  #[serde(default)]
  pub saved_posts: Vec<Url>,
  #[serde(default)]
  pub saved_comments: Vec<Url>,
  #[serde(default)]
  pub blocked_users: Vec<Url>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Your exported data.
pub struct ExportDataResponse {
  pub inbox: Vec<PostOrCommentOrPrivateMessage>,
  pub content: Vec<PostOrCommentOrPrivateMessage>,
  pub settings: UserSettingsBackup,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A response that completes successfully.
pub struct SuccessResponse {
  pub success: bool,
}

impl Default for SuccessResponse {
  fn default() -> Self {
    SuccessResponse { success: true }
  }
}
