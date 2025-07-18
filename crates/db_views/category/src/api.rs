use serde::Deserialize;
use validator::Validate;
use lemmy_db_schema::newtypes::{CategoryGroupId, CategoryId};

#[derive(Debug, Deserialize, Clone, Default, PartialEq, Eq)]
/// Create a category group
pub struct CreateCategoryGroup {
  pub title: String,
  pub sort_order: i32,
}

#[derive(Debug, Deserialize, Validate, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// Create a category group request
pub struct CreateCategoryGroupRequest {
  pub title: Option<String>,
  pub sort_order: Option<i32>,
}


#[derive(Debug, Deserialize, Clone, Default, PartialEq, Eq)]
/// Create a category.
pub struct CreateCategory {
  pub title: String,
  pub slug: Option<String>,
  pub image: Option<String>,
  pub is_new: Option<bool>,
  pub sort_order: i32,
}

#[derive(Debug, Deserialize, Validate, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// Create a category request
pub struct CreateCategoryRequest {
  pub title: Option<String>,
  pub slug: Option<String>,
  pub image: Option<String>,
  pub is_new: Option<bool>,
  pub sort_order: i32,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq, Eq)]
/// Create a subcategory.
pub struct CreateSubCategory {
  pub title: String,
  pub slug: Option<String>,
  pub image: Option<String>,
  pub is_new: Option<bool>,
  pub sort_order: i32,
  pub parent_id: CategoryId,
  pub group_id: CategoryGroupId,
}

#[derive(Debug, Deserialize, Validate, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// Create a subcategory request
pub struct CreateSubCategoryRequest {
  pub title: Option<String>,
  pub slug: Option<String>,
  pub image: Option<String>,
  pub is_new: Option<bool>,
  pub sort_order: Option<i32>,
  pub parent_id: Option<CategoryId>,
  pub group_id: Option<CategoryGroupId>,
}
