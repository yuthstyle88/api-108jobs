use crate::newtypes::{EducationId, PersonId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::education;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = education))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct Education {
    pub id: EducationId,
    pub person_id: PersonId,
    pub school_name: String,
    pub major: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = education))]
pub struct EducationInsertForm {
    pub person_id: PersonId,
    pub school_name: String,
    pub major: String,
}
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = education))]
pub struct EducationUpdateForm {
    pub school_name: Option<String>,
    pub major: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct EducationRequest {
    pub education: Vec<EducationItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct EducationItem {
    pub id: Option<EducationId>, // None for new items, Some(id) for updates
    #[serde(rename = "school", skip_serializing_if = "Option::is_none")]
    pub school_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub major: Option<String>,
    #[serde(default)]
    pub deleted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct EducationResponse {
    pub id: EducationId,
    #[serde(rename = "school")]
    pub school_name: String,
    pub major: String,
    pub created_at: DateTime<Utc>,
}

impl From<Education> for EducationResponse {
    fn from(education: Education) -> Self {
        Self {
            id: education.id,
            school_name: education.school_name,
            major: education.major,
            created_at: education.created_at,
        }
    }
}



// Individual update request structures
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct UpdateEducationRequest {
    pub id: EducationId,
    pub school_name: Option<String>,
    pub major: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct DeleteEducationsRequest {
    #[serde(rename = "educationIds")]
    pub education_ids: Vec<EducationId>,
}