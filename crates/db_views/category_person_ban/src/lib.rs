#[cfg(feature = "full")]
use diesel::{Queryable, Selectable};
use app_108jobs_db_schema::source::{category::Category, person::Person};
use serde::{Deserialize, Serialize};

#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A category person ban.
pub struct CategoryPersonBanView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category: Category,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person: Person,
}
