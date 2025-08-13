use crate::newtypes::{PersonId, SkillId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::skills;
#[cfg(feature = "full")]
use lemmy_utils::error::{FastJobError, FastJobErrorType, FastJobResult};

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = skills))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct Skills {
    pub id: i32,
    pub person_id: PersonId,
    pub skill_name: String,
    pub level_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = skills))]
pub struct SkillsInsertForm {
    pub person_id: PersonId,
    pub skill_name: String,
    pub level_id: Option<i32>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = skills))]
pub struct SkillsUpdateForm {
    pub skill_name: Option<String>,
    pub level_id: Option<Option<i32>>,
}

// A common form used for both insert and update flows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct SkillsForm {
    pub person_id: Option<PersonId>,
    pub skill_name: String,
    pub level_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct UpdateSkillRequest {
    pub id: SkillId,
    pub skill_name: Option<String>,
    pub level_id: Option<i32>, // Skill proficiency level: 1 (Beginner) to 5 (Expert)
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct SkillsRequest {
    pub skills: Vec<SkillItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct SkillItem {
    pub id: Option<SkillId>, // None for new items, Some(id) for updates
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub skill_name: Option<String>,
    #[serde(rename = "level", skip_serializing_if = "Option::is_none")]
    pub level_id: Option<i32>, // Skill proficiency level: 1 (Beginner) to 5 (Expert)
    #[serde(default)]
    pub deleted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct SkillResponse {
    pub id: i32,
    #[serde(rename = "name")]
    pub skill_name: String,
    #[serde(rename = "level")]
    pub level_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<Skills> for SkillResponse {
    fn from(skill: Skills) -> Self {
        Self {
            id: skill.id,
            skill_name: skill.skill_name,
            level_id: skill.level_id,
            created_at: skill.created_at,
            updated_at: skill.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct DeleteSkillsRequest {
    #[serde(rename = "skillIds")]
    pub skill_ids: Vec<SkillId>,
}




#[cfg(feature = "full")]
impl SkillsForm {
    fn validate(&self) -> FastJobResult<()> {
        if self.skill_name.is_empty() {
            Err(FastJobErrorType::SkillCouldntEmpty)?
        }
        if let Some(level) = self.level_id {
            if level < 1 || level > 5 {
                return Err(FastJobError::from(FastJobErrorType::InvalidField(
                    "Proficient level must from 1 to 5".to_string(),
                )));
            }
        }
        Ok(())
    }
}

#[cfg(feature = "full")]
impl TryFrom<SkillsForm> for SkillsInsertForm {
    type Error = FastJobError;

    fn try_from(value: SkillsForm) -> Result<Self, Self::Error> {
        value.validate()?;
        Ok(SkillsInsertForm {
            person_id: value.person_id.unwrap_or(PersonId(0)),
            skill_name: value.skill_name,
            level_id: value.level_id,
        })
    }
}

#[cfg(feature = "full")]
impl TryFrom<SkillsForm> for SkillsUpdateForm {
    type Error = FastJobError;

    fn try_from(value: SkillsForm) -> Result<Self, Self::Error> {
        value.validate()?;
        Ok(SkillsUpdateForm {
            skill_name: Some(value.skill_name),
            level_id: Some(value.level_id),
        })
    }
}