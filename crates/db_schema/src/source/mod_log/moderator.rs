use crate::newtypes::{
    CommentId,
    CategoryId,
    InstanceId,
    ModAddCategoryId,
    ModAddId,
    ModBanFromCategoryId,
    ModBanId,
    ModChangeCategoryVisibilityId,
    ModFeaturePostId,
    ModLockPostId,
    ModRemoveCommentId,
    ModRemoveCategoryId,
    ModRemovePostId,
    ModTransferCategoryId,
    PersonId,
    PostId,
};
use chrono::{DateTime, Utc};
use app_108jobs_db_schema_file::enums::CategoryVisibility;
#[cfg(feature = "full")]
use app_108jobs_db_schema_file::schema::{
  mod_add,
  mod_add_category,
  mod_ban,
  mod_ban_from_category,
  mod_change_category_visibility,
  mod_feature_post,
  mod_lock_post,
  mod_remove_comment,
  mod_remove_category,
  mod_remove_post,
  mod_transfer_category,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_remove_post))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator removes a post.
pub struct ModRemovePost {
  pub id: ModRemovePostId,
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub reason: Option<String>,
  pub removed: bool,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_remove_post))]
pub struct ModRemovePostForm {
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub reason: Option<String>,
  pub removed: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_lock_post))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator locks a post (prevents new comments being made).
pub struct ModLockPost {
  pub id: ModLockPostId,
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub locked: bool,
  pub published_at: DateTime<Utc>,
  pub reason: Option<String>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_lock_post))]
pub struct ModLockPostForm {
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub locked: Option<bool>,
  pub reason: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_feature_post))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator features a post on a category (pins it to the top).
pub struct ModFeaturePost {
  pub id: ModFeaturePostId,
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub featured: bool,
  pub published_at: DateTime<Utc>,
  pub is_featured_category: bool,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_feature_post))]
pub struct ModFeaturePostForm {
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub featured: Option<bool>,
  pub is_featured_category: Option<bool>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_remove_comment))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator removes a comment.
pub struct ModRemoveComment {
  pub id: ModRemoveCommentId,
  pub mod_person_id: PersonId,
  pub comment_id: CommentId,
  pub reason: Option<String>,
  pub removed: bool,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_remove_comment))]
pub struct ModRemoveCommentForm {
  pub mod_person_id: PersonId,
  pub comment_id: CommentId,
  pub reason: Option<String>,
  pub removed: Option<bool>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_remove_category))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator removes a category.
pub struct ModRemoveCategory {
  pub id: ModRemoveCategoryId,
  pub mod_person_id: PersonId,
  pub category_id: CategoryId,
  pub reason: Option<String>,
  pub removed: bool,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_remove_category))]
pub struct ModRemoveCategoryForm {
  pub mod_person_id: PersonId,
  pub category_id: CategoryId,
  pub reason: Option<String>,
  pub removed: Option<bool>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_ban_from_category))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When someone is banned from a category.
pub struct ModBanFromCategory {
  pub id: ModBanFromCategoryId,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub category_id: CategoryId,
  pub reason: Option<String>,
  pub banned: bool,
  pub expires_at: Option<DateTime<Utc>>,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_ban_from_category))]
pub struct ModBanFromCategoryForm {
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub category_id: CategoryId,
  pub reason: Option<String>,
  pub banned: Option<bool>,
  pub expires_at: Option<DateTime<Utc>>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_ban))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When someone is banned from the site.
pub struct ModBan {
  pub id: ModBanId,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub reason: Option<String>,
  pub banned: bool,
  pub expires_at: Option<DateTime<Utc>>,
  pub published_at: DateTime<Utc>,
  pub instance_id: InstanceId,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_change_category_visibility))]
pub struct ModChangeCategoryVisibilityForm {
  pub category_id: CategoryId,
  pub mod_person_id: PersonId,
  pub visibility: CategoryVisibility,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_change_category_visibility))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ModChangeCategoryVisibility {
  pub id: ModChangeCategoryVisibilityId,
  pub category_id: CategoryId,
  pub mod_person_id: PersonId,
  pub published_at: DateTime<Utc>,
  pub visibility: CategoryVisibility,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_ban))]
pub struct ModBanForm {
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub reason: Option<String>,
  pub banned: Option<bool>,
  pub expires_at: Option<DateTime<Utc>>,
  pub instance_id: InstanceId,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_add_category))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When someone is added as a category moderator.
pub struct ModAddCategory {
  pub id: ModAddCategoryId,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub category_id: CategoryId,
  pub removed: bool,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_add_category))]
pub struct ModAddCategoryForm {
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub category_id: CategoryId,
  pub removed: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_transfer_category))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator transfers a category to a new owner.
pub struct ModTransferCategory {
  pub id: ModTransferCategoryId,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub category_id: CategoryId,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_transfer_category))]
pub struct ModTransferCategoryForm {
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub category_id: CategoryId,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_add))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When someone is added as a site moderator.
pub struct ModAdd {
  pub id: ModAddId,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub removed: bool,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_add))]
pub struct ModAddForm {
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub removed: Option<bool>,
}
