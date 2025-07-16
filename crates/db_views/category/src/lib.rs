use diesel::{Queryable, Selectable};
use lemmy_db_schema::source::category::Category;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use lemmy_db_schema::source::category_group::CategoryGroup;

pub mod api;

#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A category view.
pub struct CategoryView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category: Category,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub subcategory_groups: CategoryGroup,
}
