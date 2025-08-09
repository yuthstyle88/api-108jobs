use crate::newtypes::PersonId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::work_experience;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = work_experience))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct WorkExperience {
    pub id: i32,
    pub person_id: PersonId,
    pub company_name: String,
    pub position: String,
    pub start_month: Option<String>,
    pub start_year: Option<i32>,
    pub end_month: Option<String>,
    pub end_year: Option<i32>,
    pub is_current: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = work_experience))]
pub struct WorkExperienceInsertForm {
    pub person_id: PersonId,
    pub company_name: String,
    pub position: String,
    pub start_month: Option<String>,
    pub start_year: Option<i32>,
    pub end_month: Option<String>,
    pub end_year: Option<i32>,
    #[new(default)]
    pub is_current: Option<bool>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = work_experience))]
pub struct WorkExperienceUpdateForm {
    pub company_name: Option<String>,
    pub position: Option<String>,
    pub start_month: Option<Option<String>>,
    pub start_year: Option<Option<i32>>,
    pub end_month: Option<Option<String>>,
    pub end_year: Option<Option<i32>>,
    pub is_current: Option<Option<bool>>,
}