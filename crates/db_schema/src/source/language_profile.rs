use crate::newtypes::{LanguageProfileId, PersonId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::language_profile;



#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = language_profile))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct LanguageProfile {
    pub id: LanguageProfileId,
    pub person_id: PersonId,
    pub lang: String,
    pub level_id: i32, // 1=low, 2=medium, 3=high
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = language_profile))]
pub struct LanguageProfileInsertForm {
    pub person_id: PersonId,
    pub lang: String,
    pub level_id: i32, // 1=low, 2=medium, 3=high
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = language_profile))]
pub struct LanguageProfileUpdateForm {
    pub lang: Option<String>,
    pub level_id: Option<i32>, // 1=low, 2=medium, 3=high
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct SaveLanguageProfiles {
    pub language_profiles: Vec<LanguageProfileItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct LanguageProfileRequest {
    pub id: Option<LanguageProfileId>,
    pub lang: String,
    pub level_id: i32, // 1=low, 2=medium, 3=high
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct LanguageProfileResponse {
    pub id: LanguageProfileId,
    pub lang: String,
    pub level_id: i32, // 1=low, 2=medium, 3=high
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<LanguageProfile> for LanguageProfileResponse {
    fn from(profile: LanguageProfile) -> Self {
        Self {
            id: profile.id,
            lang: profile.lang,
            level_id: profile.level_id,
            created_at: profile.created_at,
            updated_at: profile.updated_at,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct DeleteLanguageProfilesRequest {
    #[serde(rename = "languageProfileIds")]
    pub language_profile_ids: Vec<LanguageProfileId>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListLanguageProfilesResponse {
    #[serde(rename = "languageProfiles")]
    pub language_profiles: Vec<LanguageProfileResponse>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageProfileItem {
    pub id: Option<LanguageProfileId>,
    pub lang: Option<String>,
    pub level_id: Option<i32>, // 1=low, 2=medium, 3=high
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}