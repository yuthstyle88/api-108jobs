use diesel::internal::derives::multiconnection::chrono::{DateTime, Utc};
use lemmy_db_schema::newtypes::{CategoryGroupId, CategoryId};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default, PartialEq, Eq)]
/// Create a category.
pub struct CreateCategoryRequest {
  pub title: String,
  pub slug: Option<String>,
  pub image: Option<String>,
  pub active: Option<bool>,
  pub is_new: Option<bool>,
  pub sort_order: i32,
  pub parent_id: Option<CategoryId>,
  pub group_id: Option<CategoryGroupId>,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq, Eq)]
/// Create a category group
pub struct CreateCategoryGroupRequest {
  pub title: String,
  pub sort_order: i32,
}
