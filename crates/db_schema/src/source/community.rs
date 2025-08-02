use crate::{
  newtypes::{CommunityId, DbUrl, InstanceId, PersonId},
  source::placeholder_apub_url,
};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use diesel_ltree::Ltree;
use lemmy_db_schema_file::enums::{CommunityFollowerState, CommunityVisibility};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  crate::newtypes::LtreeDef,
  i_love_jesus::CursorKeysModule,
  lemmy_db_schema_file::schema::{community, community_actions},
};

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = community))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = community_keys))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A community.
#[serde(rename_all = "camelCase")]
pub struct Community {
  pub id: CommunityId,
  pub name: String,
  /// A longer title, that can contain other characters, and doesn't have to be unique.
  pub title: String,
  /// A sidebar for the community in markdown.
  pub sidebar: Option<String>,
  /// Whether the community is removed by a mod.
  pub removed: bool,
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  /// Whether the community has been deleted by its creator.
  pub deleted: bool,
  /// Whether its an NSFW community.
  pub self_promotion: bool,
  /// Whether the community is local.
  pub local: bool,
  #[serde(skip)]
  pub last_refreshed_at: DateTime<Utc>,
  /// A URL for an icon.
  pub icon: Option<DbUrl>,
  /// A URL for a banner.
  pub banner: Option<DbUrl>,
  #[cfg_attr(feature = "ts-rs", ts(skip))]
  #[serde(skip)]
  pub followers_url: Option<DbUrl>,
  #[cfg_attr(feature = "ts-rs", ts(skip))]
  #[serde(skip, default = "placeholder_apub_url")]
  pub inbox_url: DbUrl,
  /// Whether posting is restricted to mods only.
  pub posting_restricted_to_mods: bool,
  pub instance_id: InstanceId,
  /// Url where moderators collection is served over Activitypub
  #[serde(skip)]
  pub moderators_url: Option<DbUrl>,
  /// Url where featured posts collection is served over Activitypub
  #[serde(skip)]
  pub featured_url: Option<DbUrl>,
  pub visibility: CommunityVisibility,
  /// A shorter, one-line description of the site.
  pub description: Option<String>,
  #[serde(skip)]
  pub random_number: i16,
  pub subscribers: i64,
  pub posts: i64,
  pub comments: i64,
  /// The number of users with any activity in the last day.
  pub users_active_day: i64,
  /// The number of users with any activity in the last week.
  pub users_active_week: i64,
  /// The number of users with any activity in the last month.
  pub users_active_month: i64,
  /// The number of users with any activity in the last year.
  pub users_active_half_year: i64,
  #[serde(skip)]
  pub hot_rank: f64,
  pub subscribers_local: i64,
  pub report_count: i16,
  pub unresolved_report_count: i16,
  /// Number of any interactions over the last month.
  #[serde(skip)]
  pub interactions_month: i64,
  pub local_removed: bool,
  #[cfg(feature = "full")]
  #[cfg_attr(feature = "full", serde(with = "LtreeDef"))]
  pub path: Ltree,
  pub slug: String,
  pub active: bool,
  pub is_new: bool,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community))]
pub struct CommunityInsertForm {
  pub instance_id: InstanceId,
  pub name: String,
  pub title: String,
  #[new(default)]
  pub sidebar: Option<String>,
  #[new(default)]
  pub removed: Option<bool>,
  #[new(default)]
  pub published_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub updated_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub deleted: Option<bool>,
  #[new(default)]
  pub self_promotion: Option<bool>,
  #[new(default)]
  pub local: Option<bool>,
  #[new(default)]
  pub last_refreshed_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub icon: Option<DbUrl>,
  #[new(default)]
  pub banner: Option<DbUrl>,
  #[new(default)]
  pub followers_url: Option<DbUrl>,
  #[new(default)]
  pub inbox_url: Option<DbUrl>,
  #[new(default)]
  pub moderators_url: Option<DbUrl>,
  #[new(default)]
  pub featured_url: Option<DbUrl>,
  #[new(default)]
  pub posting_restricted_to_mods: Option<bool>,
  #[new(default)]
  pub visibility: Option<CommunityVisibility>,
  #[new(default)]
  pub description: Option<String>,
  #[new(default)]
  pub local_removed: Option<bool>,
  pub slug: String,
  #[new(default)]
  pub active: bool,
  #[new(default)]
  pub is_new: Option<bool>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community))]
pub struct CommunityUpdateForm {
  pub name: Option<String>,
  pub title: Option<String>,
  pub sidebar: Option<Option<String>>,
  pub removed: Option<bool>,
  pub published_at: Option<DateTime<Utc>>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
  pub deleted: Option<bool>,
  pub self_promotion: Option<bool>,
  pub local: Option<bool>,
  pub last_refreshed_at: Option<DateTime<Utc>>,
  pub icon: Option<Option<DbUrl>>,
  pub banner: Option<Option<DbUrl>>,
  pub followers_url: Option<DbUrl>,
  pub inbox_url: Option<DbUrl>,
  pub moderators_url: Option<Option<DbUrl>>,
  pub featured_url: Option<Option<DbUrl>>,
  pub posting_restricted_to_mods: Option<bool>,
  pub visibility: Option<CommunityVisibility>,
  pub description: Option<Option<String>>,
  pub local_removed: Option<bool>,
  pub slug: Option<String>,
  pub active: Option<bool>,
  pub is_new: Option<bool>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations, CursorKeysModule)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::community::Community))
)]
#[cfg_attr(feature = "full", diesel(table_name = community_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, community_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = community_actions_keys))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct CommunityActions {
  #[serde(skip)]
  pub community_id: CommunityId,
  #[serde(skip)]
  pub person_id: PersonId,
  /// When the community was followed.
  pub followed_at: Option<DateTime<Utc>>,
  /// The state of the community follow.
  pub follow_state: Option<CommunityFollowerState>,
  /// The approver of the community follow.
  #[serde(skip)]
  pub follow_approver_id: Option<PersonId>,
  /// When the community was blocked.
  pub blocked_at: Option<DateTime<Utc>>,
  /// When this user became a moderator.
  pub became_moderator_at: Option<DateTime<Utc>>,
  /// When this user received a ban.
  pub received_ban_at: Option<DateTime<Utc>>,
  /// When their ban expires.
  pub ban_expires_at: Option<DateTime<Utc>>,
}

// Create a changeset struct with explicit fields
// This avoids the complex nested type inference that was causing the compiler panic
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community))]
pub struct CommunityChangeset {
  pub(crate) title: Option<String>,
  pub(crate) sidebar: Option<Option<String>>,
  pub(crate) removed: Option<bool>,
  pub(crate) published_at: Option<DateTime<Utc>>,
  pub(crate) updated_at: Option<Option<DateTime<Utc>>>,
  pub(crate) deleted: Option<bool>,
  pub(crate) self_promotion: Option<bool>,
  pub(crate) local: Option<bool>,
  pub(crate) last_refreshed_at: Option<DateTime<Utc>>,
  pub(crate) icon: Option<Option<DbUrl>>,
  pub(crate) banner: Option<Option<DbUrl>>,
  pub(crate) followers_url: Option<DbUrl>,
  pub(crate) inbox_url: Option<DbUrl>,
  pub(crate) moderators_url: Option<Option<DbUrl>>,
  pub(crate) featured_url: Option<Option<DbUrl>>,
  pub(crate) posting_restricted_to_mods: Option<bool>,
  pub(crate) visibility: Option<CommunityVisibility>,
  pub(crate) description: Option<Option<String>>,
  pub(crate) local_removed: Option<bool>,
}