pub use lemmy_db_views_person_liked_combined::{
  ListPersonLiked,
  ListPersonLikedResponse,
  PersonLikedCombinedView,
};
pub use lemmy_db_views_person_saved_combined::{
  ListPersonSaved,
  ListPersonSavedResponse,
  PersonSavedCombinedView,
};
pub use lemmy_db_views_site::api::{
  DeleteAccount,
  ListPersonHidden,
  ListPersonHiddenResponse,
  ListPersonRead,
  ListPersonReadResponse,
  MyUserInfo,
  SaveUserSettings,
};

pub mod auth {
  pub use lemmy_db_schema::source::login_token::LoginToken;
  pub use lemmy_db_views_registration_applications::api::Register;
  pub use lemmy_db_views_site::api::{
    CaptchaResponse,
    ChangePassword,
    GenerateTotpSecretResponse,
    GetCaptchaResponse,
    ListLoginsResponse,
    Login,
    LoginResponse,
    PasswordChangeAfterReset,
    PasswordReset,
    ResendVerificationEmail,
    UpdateTotp,
    UpdateTotpResponse,
    VerifyEmail,
  };
}

use serde::{Deserialize, Serialize};
use lemmy_db_schema::newtypes::{CertificateId, EducationId, SkillId, WorkExperienceId};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct EducationItem {
    pub id: Option<EducationId>, // None for new items, Some(id) for updates
    pub school_name: String,
    pub major: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct EducationRequest {
    pub educations: Vec<EducationItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct WorkExperienceItem {
    pub id: Option<WorkExperienceId>, // None for new items, Some(id) for updates
    pub company_name: String,
    pub position: String,
    pub start_month: Option<String>,
    pub start_year: Option<i32>,
    pub end_month: Option<String>,
    pub end_year: Option<i32>,
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
pub struct SkillItem {
    pub id: Option<SkillId>, // None for new items, Some(id) for updates
    pub skill_name: String,
    pub level_id: Option<i32>, // Skill proficiency level: 1 (Beginner) to 5 (Expert)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct SkillsRequest {
    pub skills: Vec<SkillItem>,
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
pub struct UpdateWorkExperienceRequest {
    pub id: WorkExperienceId,
    pub company_name: Option<String>,
    pub position: Option<String>,
    pub start_month: Option<String>,
    pub start_year: Option<i32>,
    pub end_month: Option<String>,
    pub end_year: Option<i32>,
    pub is_current: Option<bool>,
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
pub struct UpdateCertificateRequest {
    pub id: CertificateId,
    pub name: String,
    pub achieved_date: Option<String>, // Date as string in YYYY-MM-DD format
    pub expires_date: Option<String>,  // Date as string in YYYY-MM-DD format
    pub url: Option<String>,
}

// Delete request structures
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct DeleteItemRequest<T> {
    pub id: T,
}
