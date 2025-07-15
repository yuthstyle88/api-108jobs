use crate::newtypes::CategoryGroupId;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::category_group;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = category_group))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct CategoryGroup {
  pub id: CategoryGroupId,
  pub title: String,
  pub sort_order: i32,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}