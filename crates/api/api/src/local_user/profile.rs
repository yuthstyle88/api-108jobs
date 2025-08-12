use actix_web::web::{Data, Json, Path};
use lemmy_api_common::account::DeleteItemRequest;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::{CertificateId, EducationId, LanguageProfileId, SkillId, WorkExperienceId};
use lemmy_db_schema::source::{
  certificates::{CertificateView, Certificates, CertificatesInsertForm, CertificatesRequest, CertificatesUpdateForm, UpdateCertificateRequestItem},
  education::{Education, EducationInsertForm, EducationRequest, EducationUpdateForm, UpdateEducationRequest},
  language_profile::{LanguageProfile, LanguageProfileInsertForm, LanguageProfileResponse, SaveLanguageProfiles, ListLanguageProfilesResponse, LanguageProfileUpdateForm},
  skills::{Skills, SkillsInsertForm, SkillsRequest, SkillsUpdateForm, UpdateSkillRequest},
  work_experience::{WorkExperience, WorkExperienceInsertForm, WorkExperienceRequest, WorkExperienceUpdateForm, UpdateWorkExperienceRequest},
};
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProfileData {
  Education(EducationRequest),
  WorkExperience(WorkExperienceRequest),
  Skills(SkillsRequest),
  Certificates(CertificatesRequest),
  LanguageProfiles(SaveLanguageProfiles),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProfileUpdateData {
  Education(UpdateEducationRequest),
  WorkExperience(UpdateWorkExperienceRequest),
  Skills(UpdateSkillRequest),
  Certificates(UpdateCertificateRequestItem),
  LanguageProfiles { id: LanguageProfileId, lang: String, level_name: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProfileDeleteData {
  Education(EducationId),
  WorkExperience(WorkExperienceId),
  Skills(SkillId),
  Certificates(CertificateId),
  LanguageProfiles(LanguageProfileId),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProfileResponse {
  Education(Vec<Education>),
  WorkExperience(Vec<WorkExperience>),
  Skills(Vec<Skills>),
  Certificates(Vec<CertificateView>),
  LanguageProfiles(ListLanguageProfilesResponse),
}

pub async fn save_profile(
  data: Json<ProfileData>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ProfileResponse>> {
  let person_id = local_user_view.person.id;

  match data.into_inner() {
    ProfileData::Education(edu_req) => {
      let mut saved_educations = Vec::new();
      for edu in &edu_req.educations {
        let saved = match edu.id {
          Some(id) => {
            let form = EducationUpdateForm {
              school_name: Some(edu.school_name.clone()),
              major: Some(edu.major.clone()),
            };
            Education::update(&mut context.pool(), id, &form).await?
          }
          None => {
            let form = EducationInsertForm::new(
              person_id,
              edu.school_name.clone(),
              edu.major.clone(),
            );
            Education::create(&mut context.pool(), &form).await?
          }
        };
        saved_educations.push(saved);
      }
      Ok(Json(ProfileResponse::Education(saved_educations)))
    }
    ProfileData::WorkExperience(work_req) => {
      let mut saved_work = Vec::new();
      for work in &work_req.work_experiences {
        let saved = match work.id {
          Some(id) => {
            let form = WorkExperienceUpdateForm {
              company_name: Some(work.company_name.clone()),
              position: Some(work.position.clone()),
              start_date: Some(work.start_date),
              end_date: work.end_date,
              description: work.description.clone(),
            };
            WorkExperience::update(&mut context.pool(), id, &form).await?
          }
          None => {
            let form = WorkExperienceInsertForm::new(
              person_id,
              work.company_name.clone(),
              work.position.clone(),
              work.start_date,
              work.end_date,
              work.description.clone(),
            );
            WorkExperience::create(&mut context.pool(), &form).await?
          }
        };
        saved_work.push(saved);
      }
      Ok(Json(ProfileResponse::WorkExperience(saved_work)))
    }
    ProfileData::Skills(skills_req) => {
      let mut saved_skills = Vec::new();
      for skill in &skills_req.skills {
        let saved = match skill.id {
          Some(id) => {
            let form = SkillsUpdateForm {
              name: Some(skill.name.clone()),
              level: Some(skill.level.clone()),
            };
            Skills::update(&mut context.pool(), id, &form).await?
          }
          None => {
            let form = SkillsInsertForm::new(
              person_id,
              skill.name.clone(),
              skill.level.clone(),
            );
            Skills::create(&mut context.pool(), &form).await?
          }
        };
        saved_skills.push(saved);
      }
      Ok(Json(ProfileResponse::Skills(saved_skills)))
    }
    ProfileData::Certificates(cert_req) => {
      let mut saved_certificates = Vec::new();
      for cert in cert_req.certificates.clone() {
        let saved = match cert.id {
          Some(id) => {
            let form: CertificatesUpdateForm = cert.into();
            Certificates::update(&mut context.pool(), id, &form).await?
          }
          None => {
            let form = CertificatesInsertForm::new(
              person_id,
              cert.name.clone(),
              cert.achieved_date,
              cert.expires_date,
              cert.url.clone(),
            );
            Certificates::create(&mut context.pool(), &form).await?
          }
        };
        saved_certificates.push(saved);
      }
      let certificate_views = Certificates::query_with_filters(&mut context.pool(), Some(person_id)).await?;
      Ok(Json(ProfileResponse::Certificates(certificate_views)))
    }
    ProfileData::LanguageProfiles(lang_req) => {
      let saved_profiles = LanguageProfile::save_language_profiles(
        &mut context.pool(),
        person_id,
        lang_req.language_profiles,
      ).await?;
      Ok(Json(ProfileResponse::LanguageProfiles(ListLanguageProfilesResponse {
        language_profiles: saved_profiles,
      })))
    }
  }
}

pub async fn list_profile(
  profile_type: Path<String>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ProfileResponse>> {
  let person_id = local_user_view.person.id;

  match profile_type.as_str() {
    "education" => {
      let educations = Education::read_by_person_id(&mut context.pool(), person_id).await?;
      Ok(Json(ProfileResponse::Education(educations)))
    }
    "work-experience" => {
      let work_experiences = WorkExperience::read_by_person_id(&mut context.pool(), person_id).await?;
      Ok(Json(ProfileResponse::WorkExperience(work_experiences)))
    }
    "skills" => {
      let skills = Skills::read_by_person_id(&mut context.pool(), person_id).await?;
      Ok(Json(ProfileResponse::Skills(skills)))
    }
    "certificates" => {
      let certificates = Certificates::query_with_filters(&mut context.pool(), Some(person_id)).await?;
      Ok(Json(ProfileResponse::Certificates(certificates)))
    }
    "language-profiles" => {
      let language_profiles = LanguageProfile::list_for_person(&mut context.pool(), person_id).await?;
      Ok(Json(ProfileResponse::LanguageProfiles(ListLanguageProfilesResponse {
        language_profiles,
      })))
    }
    _ => Err(lemmy_utils::error::FastJobError::invalid_input("Invalid profile type")),
  }
}

pub async fn update_profile(
  data: Json<ProfileUpdateData>,
  context: Data<FastJobContext>,
  _local_user_view: LocalUserView,
) -> FastJobResult<Json<String>> {
  match data.into_inner() {
    ProfileUpdateData::Education(edu_req) => {
      let form = EducationUpdateForm {
        school_name: edu_req.school_name.clone(),
        major: edu_req.major.clone(),
      };
      Education::update(&mut context.pool(), edu_req.id, &form).await?;
      Ok(Json("Education updated successfully".to_string()))
    }
    ProfileUpdateData::WorkExperience(work_req) => {
      let form = WorkExperienceUpdateForm {
        company_name: Some(work_req.company_name),
        position: Some(work_req.position),
        start_date: Some(work_req.start_date),
        end_date: work_req.end_date,
        description: work_req.description,
      };
      WorkExperience::update(&mut context.pool(), work_req.id, &form).await?;
      Ok(Json("Work experience updated successfully".to_string()))
    }
    ProfileUpdateData::Skills(skill_req) => {
      let form = SkillsUpdateForm {
        name: Some(skill_req.name),
        level: Some(skill_req.level),
      };
      Skills::update(&mut context.pool(), skill_req.id, &form).await?;
      Ok(Json("Skill updated successfully".to_string()))
    }
    ProfileUpdateData::Certificates(cert_req) => {
      if let Some(id) = cert_req.id {
        let form: CertificatesUpdateForm = cert_req.into();
        Certificates::update(&mut context.pool(), id, &form).await?;
      }
      Ok(Json("Certificate updated successfully".to_string()))
    }
    ProfileUpdateData::LanguageProfiles { id, lang, level_name } => {
      let form = LanguageProfileUpdateForm {
        lang: Some(lang),
        level_name: Some(level_name),
        updated_at: Some(chrono::Utc::now()),
      };
      LanguageProfile::update(&mut context.pool(), id, &form).await?;
      Ok(Json("Language profile updated successfully".to_string()))
    }
  }
}

pub async fn delete_profile(
  data: Json<ProfileDeleteData>,
  context: Data<FastJobContext>,
  _local_user_view: LocalUserView,
) -> FastJobResult<Json<String>> {
  match data.into_inner() {
    ProfileDeleteData::Education(id) => {
      Education::delete(&mut context.pool(), id).await?;
      Ok(Json("Education deleted successfully".to_string()))
    }
    ProfileDeleteData::WorkExperience(id) => {
      WorkExperience::delete(&mut context.pool(), id).await?;
      Ok(Json("Work experience deleted successfully".to_string()))
    }
    ProfileDeleteData::Skills(id) => {
      Skills::delete(&mut context.pool(), id).await?;
      Ok(Json("Skill deleted successfully".to_string()))
    }
    ProfileDeleteData::Certificates(id) => {
      Certificates::delete(&mut context.pool(), id).await?;
      Ok(Json("Certificate deleted successfully".to_string()))
    }
    ProfileDeleteData::LanguageProfiles(id) => {
      LanguageProfile::delete(&mut context.pool(), id).await?;
      Ok(Json("Language profile deleted successfully".to_string()))
    }
  }
}