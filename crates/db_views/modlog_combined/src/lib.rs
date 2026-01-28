use app_108jobs_db_schema::source::{
  comment::Comment,
  category::Category,
  instance::Instance,
  mod_log::{
    admin::{
      AdminAllowInstance,
      AdminBlockInstance,
      AdminPurgeComment,
      AdminPurgeCategory,
      AdminPurgePerson,
      AdminPurgePost,
    },
    moderator::{
      ModAdd,
      ModAddCategory,
      ModBan,
      ModBanFromCategory,
      ModChangeCategoryVisibility,
      ModFeaturePost,
      ModLockPost,
      ModRemoveComment,
      ModRemoveCategory,
      ModRemovePost,
      ModTransferCategory,
    },
  },
  person::Person,
  post::Post,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{dsl::Nullable, NullableExpressionMethods, Queryable, Selectable},
  app_108jobs_db_schema::{utils::queries::person1_select, Person1AliasAllColumnsTuple},
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When someone is added as a category moderator.
#[serde(rename_all = "camelCase")]
pub struct ModAddCategoryView {
  pub mod_add_category: ModAddCategory,
  pub moderator: Option<Person>,
  pub category: Category,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When someone is added as a site moderator.
#[serde(rename_all = "camelCase")]
pub struct ModAddView {
  pub mod_add: ModAdd,
  pub moderator: Option<Person>,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When someone is banned from a category.
#[serde(rename_all = "camelCase")]
pub struct ModBanFromCategoryView {
  pub mod_ban_from_category: ModBanFromCategory,
  pub moderator: Option<Person>,
  pub category: Category,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When someone is banned from the site.
#[serde(rename_all = "camelCase")]
pub struct ModBanView {
  pub mod_ban: ModBan,
  pub moderator: Option<Person>,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When the visibility of a category is changed
#[serde(rename_all = "camelCase")]
pub struct ModChangeCategoryVisibilityView {
  pub mod_change_category_visibility: ModChangeCategoryVisibility,
  pub moderator: Option<Person>,
  pub category: Category,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator locks a post (prevents new comments being made).
#[serde(rename_all = "camelCase")]
pub struct ModLockPostView {
  pub mod_lock_post: ModLockPost,
  pub moderator: Option<Person>,
  pub other_person: Person,
  pub post: Post,
  /// Category is optional for posts without categories (e.g., delivery posts)
  pub category: Option<Category>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator removes a comment.
#[serde(rename_all = "camelCase")]
pub struct ModRemoveCommentView {
  pub mod_remove_comment: ModRemoveComment,
  pub moderator: Option<Person>,
  pub other_person: Person,
  pub comment: Comment,
  pub post: Post,
  /// Category is optional for posts without categories (e.g., delivery posts)
  pub category: Option<Category>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator removes a category.
#[serde(rename_all = "camelCase")]
pub struct ModRemoveCategoryView {
  pub mod_remove_category: ModRemoveCategory,
  pub moderator: Option<Person>,
  pub category: Category,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator removes a post.
pub struct ModRemovePostView {
  pub mod_remove_post: ModRemovePost,
  pub moderator: Option<Person>,
  pub other_person: Person,
  pub post: Post,
  /// Category is optional for posts without categories (e.g., delivery posts)
  pub category: Option<Category>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator features a post on a category (pins it to the top).
pub struct ModFeaturePostView {
  pub mod_feature_post: ModFeaturePost,
  pub moderator: Option<Person>,
  pub other_person: Person,
  pub post: Post,
  /// Category is optional for posts without categories (e.g., delivery posts)
  pub category: Option<Category>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator transfers a category to a new owner.
pub struct ModTransferCategoryView {
  pub mod_transfer_category: ModTransferCategory,
  pub moderator: Option<Person>,
  pub category: Category,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When an admin purges a comment.
pub struct AdminPurgeCommentView {
  pub admin_purge_comment: AdminPurgeComment,
  pub admin: Option<Person>,
  pub post: Post,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When an admin purges a category.
pub struct AdminPurgeCategoryView {
  pub admin_purge_category: AdminPurgeCategory,
  pub admin: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When an admin purges a person.
pub struct AdminPurgePersonView {
  pub admin_purge_person: AdminPurgePerson,
  pub admin: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When an admin purges a post.
pub struct AdminPurgePostView {
  pub admin_purge_post: AdminPurgePost,
  pub admin: Option<Person>,
  /// Category is optional for posts without categories (e.g., delivery posts)
  pub category: Option<Category>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When an admin purges a post.
pub struct AdminBlockInstanceView {
  pub admin_block_instance: AdminBlockInstance,
  pub instance: Instance,
  pub admin: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When an admin purges a post.
pub struct AdminAllowInstanceView {
  pub admin_allow_instance: AdminAllowInstance,
  pub instance: Instance,
  pub admin: Option<Person>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined modlog view
pub(crate) struct ModlogCombinedViewInternal {
  // Specific
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_allow_instance: Option<AdminAllowInstance>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_block_instance: Option<AdminBlockInstance>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_purge_comment: Option<AdminPurgeComment>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_purge_category: Option<AdminPurgeCategory>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_purge_person: Option<AdminPurgePerson>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_purge_post: Option<AdminPurgePost>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_add: Option<ModAdd>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_add_category: Option<ModAddCategory>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_ban: Option<ModBan>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_ban_from_category: Option<ModBanFromCategory>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_feature_post: Option<ModFeaturePost>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_change_category_visibility: Option<ModChangeCategoryVisibility>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_lock_post: Option<ModLockPost>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_remove_comment: Option<ModRemoveComment>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_remove_category: Option<ModRemoveCategory>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_remove_post: Option<ModRemovePost>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_transfer_category: Option<ModTransferCategory>,
  // Specific fields

  // Shared
  #[cfg_attr(feature = "full", diesel(embed))]
  pub moderator: Option<Person>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<Person1AliasAllColumnsTuple>,
      select_expression = person1_select().nullable()
    )
  )]
  pub other_person: Option<Person>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub instance: Option<Instance>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category: Option<Category>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Option<Post>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment: Option<Comment>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
// Use serde's internal tagging, to work easier with javascript libraries
#[serde(tag = "type_")]
pub enum ModlogCombinedView {
  AdminAllowInstance(AdminAllowInstanceView),
  AdminBlockInstance(AdminBlockInstanceView),
  AdminPurgeComment(AdminPurgeCommentView),
  AdminPurgeCategory(AdminPurgeCategoryView),
  AdminPurgePerson(AdminPurgePersonView),
  AdminPurgePost(AdminPurgePostView),
  ModAdd(ModAddView),
  ModAddCategory(ModAddCategoryView),
  ModBan(ModBanView),
  ModBanFromCategory(ModBanFromCategoryView),
  ModFeaturePost(ModFeaturePostView),
  ModChangeCategoryVisibility(ModChangeCategoryVisibilityView),
  ModLockPost(ModLockPostView),
  ModRemoveComment(ModRemoveCommentView),
  ModRemoveCategory(ModRemoveCategoryView),
  ModRemovePost(ModRemovePostView),
  ModTransferCategory(ModTransferCategoryView),
}
