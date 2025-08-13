use crate::newtypes::{ LanguageProfileId, PersonId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::io::Write;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::language_profile;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(diesel::expression::AsExpression, diesel::FromSqlRow))]
#[cfg_attr(feature = "full", diesel(sql_type = diesel::sql_types::Text))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum LanguageLevel {
  /// Native or bilingual proficiency
  Native,
  /// Near-native proficiency (C2)
  NearNative,
  /// Advanced proficiency (C1)
  Advanced,
  /// Upper intermediate proficiency (B2)
  UpperIntermediate,
  /// Intermediate proficiency (B1)
  Intermediate,
  /// Pre-intermediate proficiency (A2)
  PreIntermediate,
  /// Beginner proficiency (A1)
  Beginner,
}

impl std::fmt::Display for LanguageLevel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      LanguageLevel::Native => write!(f, "Native"),
      LanguageLevel::NearNative => write!(f, "Near Native (C2)"),
      LanguageLevel::Advanced => write!(f, "Advanced (C1)"),
      LanguageLevel::UpperIntermediate => write!(f, "Upper Intermediate (B2)"),
      LanguageLevel::Intermediate => write!(f, "Intermediate (B1)"),
      LanguageLevel::PreIntermediate => write!(f, "Pre-Intermediate (A2)"),
      LanguageLevel::Beginner => write!(f, "Beginner (A1)"),
    }
  }
}

#[cfg(feature = "full")]
impl diesel::serialize::ToSql<diesel::sql_types::Text, diesel::pg::Pg> for LanguageLevel {
  fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>) -> diesel::serialize::Result {
    let s = match self {
      LanguageLevel::Native => "native",
      LanguageLevel::NearNative => "near_native",
      LanguageLevel::Advanced => "advanced",
      LanguageLevel::UpperIntermediate => "upper_intermediate",
      LanguageLevel::Intermediate => "intermediate",
      LanguageLevel::PreIntermediate => "pre_intermediate",
      LanguageLevel::Beginner => "beginner",
    };
    out.write_all(s.as_bytes())?;
    Ok(diesel::serialize::IsNull::No)
  }
}

#[cfg(feature = "full")]
impl diesel::deserialize::FromSql<diesel::sql_types::Text, diesel::pg::Pg> for LanguageLevel {
  fn from_sql(bytes: diesel::backend::RawValue<diesel::pg::Pg>) -> diesel::deserialize::Result<Self> {
    let s = std::str::from_utf8(bytes.as_bytes())?;
    match s {
      "native" => Ok(LanguageLevel::Native),
      "near_native" => Ok(LanguageLevel::NearNative),
      "advanced" => Ok(LanguageLevel::Advanced),
      "upper_intermediate" => Ok(LanguageLevel::UpperIntermediate),
      "intermediate" => Ok(LanguageLevel::Intermediate),
      "pre_intermediate" => Ok(LanguageLevel::PreIntermediate),
      "beginner" => Ok(LanguageLevel::Beginner),
      _ => Err("Invalid language level".into()),
    }
  }
}

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
    pub level_name: LanguageLevel,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct LanguageProfileItem {
    pub id: Option<LanguageProfileId>, // None for new items, Some(id) for updates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level_name: Option<LanguageLevel>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = language_profile))]
pub struct LanguageProfileInsertForm {
    pub person_id: PersonId,
    pub lang: String,
    pub level_name: LanguageLevel,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = language_profile))]
pub struct LanguageProfileUpdateForm {
    pub lang: Option<String>,
    pub level_name: Option<LanguageLevel>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct SaveLanguageProfiles {
    #[serde(rename = "languageProfiles")]
    pub language_profiles: Vec<LanguageProfileRequest>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct LanguageProfileRequest {
    pub id: Option<LanguageProfileId>,
    pub lang: String,
    pub level_name: LanguageLevel,
    #[serde(default)]
    pub deleted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct LanguageProfileResponse {
    pub id: LanguageProfileId,
    pub lang: String,
    #[serde(rename = "level")]
    pub level_name: LanguageLevel,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<LanguageProfile> for LanguageProfileResponse {
    fn from(profile: LanguageProfile) -> Self {
        Self {
            id: profile.id,
            lang: profile.lang,
            level_name: profile.level_name,
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