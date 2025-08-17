use actix_web::web::{Data, Json, Path};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::{CertificateId, EducationId, LanguageProfileId, SkillId, WorkExperienceId};
use lemmy_db_schema::source::identity_card::{IdentityCard, IdentityCardUpdateForm};
use lemmy_db_schema::source::person::{Person, PersonUpdateForm, SaveUserProfileForm};
use lemmy_db_schema::source::{
  certificates::{CertificateView, Certificates, CertificatesRequest, UpdateCertificateRequestItem},
  education::{Education, EducationRequest, UpdateEducationRequest},
  language_profile::{LanguageProfile, ListLanguageProfilesResponse, SaveLanguageProfiles},
  skills::{Skills, SkillsRequest, UpdateSkillRequest},
  work_experience::{UpdateWorkExperienceRequest, WorkExperience, WorkExperienceRequest},
};
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobError, FastJobErrorType, FastJobResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProfileData {
  Person(SaveUserProfileForm),
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
pub struct  ProfileResponse;
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProfileListResponse {
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
    ProfileData::Person(update_person) => {
      let person_form = update_person.person;
      let id_card = update_person.id_card;
      let form = PersonUpdateForm{
        display_name: Some(person_form.display_name.clone()),
        name: person_form.name.clone(),
        avatar: Some(person_form.avatar.clone()),
        bio: Some(person_form.bio.clone()),
        ..Default::default()
      };
      let _ = Person::update(&mut context.pool(),person_id, &form).await?;
      let identity_card_id = local_user_view.person.identity_card_id;
      let form = IdentityCardUpdateForm{
          date_of_birth: id_card.date_of_birth, 
        ..Default::default()
      };
      let _ = IdentityCard::update(&mut context.pool(),identity_card_id, &form).await?;
      Ok(Json(ProfileResponse))
    }
    ProfileData::Education(edu_req) => {
      let _ = Education::save_education_list(&mut context.pool(),person_id, &edu_req.education).await?;
      Ok(Json(ProfileResponse))
    }
    ProfileData::WorkExperience(work_req) => {
      let _ =   WorkExperience::save_work_experience_list(&mut context.pool(), person_id, &work_req.work_experience).await?;
      Ok(Json(ProfileResponse))
    }
    ProfileData::Skills(skills_req) => {
      let _ = Skills::save_skills_list(&mut context.pool(), person_id, &skills_req.skills).await?;
      Ok(Json(ProfileResponse))
    }
    ProfileData::Certificates(cert_req) => {
      let _ = Certificates::save_certificate_list(&mut context.pool(),person_id, &cert_req.certificates).await?;
      Ok(Json(ProfileResponse))
    }
    ProfileData::LanguageProfiles(lang_req) => {
      let _ = LanguageProfile::save_language_profile_list(&mut context.pool(), person_id, &lang_req.language_profiles).await?;
      Ok(Json(ProfileResponse))
    }
  }
}

pub async fn list_profile(
  profile_type: Path<String>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ProfileListResponse>> {
  let person_id = local_user_view.person.id;

  match profile_type.as_str() {
    "education" => {
      let educations = Education::read_by_person_id(&mut context.pool(), person_id).await?;
      Ok(Json(ProfileListResponse::Education(educations)))
    }
    "work-experience" => {
      let work_experiences = WorkExperience::read_by_person_id(&mut context.pool(), person_id).await?;
      Ok(Json(ProfileListResponse::WorkExperience(work_experiences)))
    }
    "skills" => {
      let skills = Skills::read_by_person_id(&mut context.pool(), person_id).await?;
      Ok(Json(ProfileListResponse::Skills(skills)))
    }
    "certificates" => {
      let certificates = Certificates::query_with_filters(&mut context.pool(), Some(person_id)).await?;
      Ok(Json(ProfileListResponse::Certificates(certificates)))
    }
    "language-profiles" => {
      let language_profiles = LanguageProfile::list_for_person(&mut context.pool(), person_id).await?;
      Ok(Json(ProfileListResponse::LanguageProfiles(ListLanguageProfilesResponse {
        language_profiles,
      })))
    }
    _ => Err(FastJobError::from(FastJobErrorType::NotFound)),
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