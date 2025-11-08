use crate::newtypes::DbUrl;
use url::Url;

#[cfg(feature = "full")]
pub mod activity;
pub mod actor_language;
pub mod captcha_answer;
pub mod combined;
pub mod comment;
pub mod comment_reply;
pub mod comment_report;
pub mod community;
pub mod community_report;
pub mod custom_emoji;
pub mod custom_emoji_keyword;
pub mod email_verification;
pub mod images;
pub mod instance;
pub mod keyword_block;
pub mod language;
pub mod local_site;
pub mod local_site_rate_limit;
pub mod local_site_url_blocklist;
pub mod local_user;
pub mod login_token;
pub mod mod_log;
pub mod oauth_account;
pub mod oauth_provider;
pub mod password_reset_request;
pub mod person;
pub mod person_comment_mention;
pub mod person_post_mention;
pub mod post;
pub mod post_report;
pub mod post_tag;
pub mod registration_application;
pub mod secret;
pub mod site;
pub mod tag;
pub mod tagline;
pub mod wallet;
pub mod billing;
pub mod bank;
pub mod user_bank_account;
pub mod chat_room;
pub mod chat_message;
pub mod coin;
pub mod workflow;
pub mod job_budget_plan;
pub mod chat_participant;
pub mod user_review;
pub mod last_read;
pub mod pending_sender_ack;
pub mod top_up_request;
pub mod withdraw_request;

/// Default value for columns like [community::Community.inbox_url] which are marked as serde(skip).
///
/// This is necessary so they can be successfully deserialized from API responses, even though the
/// value is not sent by Lemmy. Necessary for crates which rely on Rust API such as
/// lemmy-stats-crawler.
#[allow(clippy::expect_used)]
fn placeholder_apub_url() -> DbUrl {
  DbUrl(Box::new(
    Url::parse("http://example.com").expect("parse placeholder url"),
  ))
}
