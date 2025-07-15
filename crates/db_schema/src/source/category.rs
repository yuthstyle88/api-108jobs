use crate::newtypes::{CategoryGroupId, CategoryId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {crate::newtypes::LtreeDef, diesel_ltree::Ltree, lemmy_db_schema_file::schema::category};

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable)
)]
#[cfg_attr(feature = "full", diesel(table_name = category))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct Category {
  pub id: CategoryId,
  pub group_id: CategoryGroupId,
  #[cfg(feature = "full")]
  #[cfg_attr(feature = "full", serde(with = "LtreeDef"))]
  pub path: Ltree,
  pub title: String,
  pub subtitle: Option<String>,
  pub slug: String,
  pub image: Option<String>,
  pub active: bool,
  pub is_new: Option<bool>,
  pub sort_order: i32,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(
  feature = "full",
  derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
#[cfg_attr(feature = "full", diesel(table_name = category))]
pub struct CategoryInsertForm {
  pub group_id: CategoryGroupId,
  pub title: String,
  pub subtitle: Option<String>,
  pub slug: String,
  pub image: Option<String>,
  pub active: Option<bool>,
  pub is_new: Option<bool>,
  pub sort_order: i32,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset, Serialize, Deserialize))]
#[cfg_attr(feature = "full", diesel(table_name = category))]
pub struct CategoryUpdateForm {
  pub title: Option<String>,
  pub slug: Option<String>,
  pub image: Option<String>,
  pub active: Option<bool>,
  pub is_new: Option<bool>,
  pub sort_order: Option<i32>,
  pub updated_at: Option<DateTime<Utc>>,
}