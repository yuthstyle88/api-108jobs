use app_108jobs_db_schema::source::{
  category::{Category, CategoryActions},
  instance::InstanceActions,
  tag::TagsView,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  app_108jobs_db_schema::utils::queries::{category_post_tags_fragment, local_user_category_can_mod},
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;
pub mod validator;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A category view.
#[serde(rename_all = "camelCase")]
pub struct CategoryView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category: Category,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category_actions: Option<CategoryActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = local_user_category_can_mod()
    )
  )]
  pub can_mod: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = category_post_tags_fragment()
    )
  )]
  pub post_tags: TagsView,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Category node.
pub struct CategoryNodeView {
  pub category: Category,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub children: Vec<CategoryNodeView>,
}
