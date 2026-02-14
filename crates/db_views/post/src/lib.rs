use chrono::{DateTime, Utc};
use app_108jobs_db_schema::source::{
  category::{Category, CategoryActions},
  images::ImageDetails,
  instance::InstanceActions,
  person::{Person, PersonActions},
  post::{Post, PostActions},
  tag::TagsView,
};
use serde::{Deserialize, Serialize};
#[cfg(test)]
pub mod db_perf;
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  app_108jobs_db_schema::utils::queries::{
    creator_banned_from_category,
    creator_banned_within_category,
  },
  app_108jobs_db_schema::utils::queries::{
    creator_is_moderator,
    local_user_can_mod_post,
    post_creator_is_admin,
    post_tags_fragment,
  },
};
use app_108jobs_db_schema::newtypes::{Coin, LanguageId, PersonId, PostId};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;
pub mod validator;
pub mod logistics;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A post view.
#[serde(rename_all = "camelCase")]
pub struct PostView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Post,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  /// Category is optional for delivery posts (which rely on post_kind for distinction)
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category: Option<Category>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub image_details: Option<ImageDetails>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category_actions: Option<CategoryActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post_actions: Option<PostActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = post_creator_is_admin()
    )
  )]
  pub creator_is_admin: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = post_tags_fragment()
    )
  )]
  pub tags: TagsView,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = local_user_can_mod_post()
    )
  )]
  pub can_mod: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_banned_within_category()
    )
  )]
  pub creator_banned: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_is_moderator()
    )
  )]
  pub creator_is_moderator: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_banned_from_category()
    )
  )]
  pub creator_banned_from_category: bool,
}

/// View only
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[serde(rename_all = "camelCase")]
pub struct PostPreview {
  pub id: PostId,
  pub name: String,
  pub budget: Coin,
  pub language_id: LanguageId,
  pub deadline: Option<DateTime<Utc>>,
  pub creator_id: PersonId,
}

