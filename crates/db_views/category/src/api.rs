use lemmy_db_schema::newtypes::CategoryId;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a category.
pub struct CreateCategoryRequest {
  pub title: String,
  pub slug: Option<String>,
  pub image: Option<String>,
  pub active: Option<bool>,
  pub is_new: Option<bool>,
  pub sort_order: i32,
  pub parent_id: Option<CategoryId>,
}
