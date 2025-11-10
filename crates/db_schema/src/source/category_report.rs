use crate::newtypes::{CategoryId, CategoryReportId, DbUrl, PersonId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::category_report;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::category::Category))
)]
#[cfg_attr(feature = "full", diesel(table_name = category_report))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A comment report.
#[serde(rename_all = "camelCase")]
pub struct CategoryReport {
  pub id: CategoryReportId,
  pub creator_id: PersonId,
  pub category_id: CategoryId,
  pub original_category_name: String,
  pub original_category_title: String,
  pub original_category_description: Option<String>,
  pub original_category_sidebar: Option<String>,
  pub original_category_icon: Option<String>,
  pub original_category_banner: Option<String>,
  pub reason: String,
  pub resolved: bool,
  pub resolver_id: Option<PersonId>,
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = category_report))]
pub struct CategoryReportForm {
  pub creator_id: PersonId,
  pub category_id: CategoryId,
  pub original_category_name: String,
  pub original_category_title: String,
  pub original_category_description: Option<String>,
  pub original_category_sidebar: Option<String>,
  pub original_category_icon: Option<DbUrl>,
  pub original_category_banner: Option<DbUrl>,
  pub reason: String,
}
