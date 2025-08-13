use crate::newtypes::{PersonId, WorkExperienceId};
use chrono::{DateTime, NaiveDate, Utc};
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
    pub id: WorkExperienceId,
    pub person_id: PersonId,
    pub company_name: String,
    pub position: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
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
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    #[new(default)]
    pub is_current: Option<bool>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = work_experience))]
pub struct WorkExperienceUpdateForm {
    pub company_name: Option<String>,
    pub position: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<Option<NaiveDate>>,
    pub is_current: Option<Option<bool>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkExperienceRequest {
    pub id: WorkExperienceId,
    pub company_name: Option<String>,
    pub position: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub is_current: Option<bool>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct WorkExperienceRequest {
    pub work_experiences: Vec<WorkExperienceItem>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct WorkExperienceItem {
    pub id: Option<WorkExperienceId>, // None for new items, Some(id) for updates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub is_current: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct WorkExperienceResponse {
    pub id: WorkExperienceId,
    pub company_name: String,
    pub position: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub is_current: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<WorkExperience> for WorkExperienceResponse {
    fn from(experience: WorkExperience) -> Self {
        Self {
            id: experience.id,
            company_name: experience.company_name,
            position: experience.position,
            start_date: experience.start_date,
            end_date: experience.end_date,
            is_current: experience.is_current,
            created_at: experience.created_at,
            updated_at: experience.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct DeleteWorkExperiencesRequest {
    #[serde(rename = "workExperienceIds")]
    pub work_experience_ids: Vec<WorkExperienceId>,
}
