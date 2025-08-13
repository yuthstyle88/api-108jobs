use actix_web::web::{Data, Json};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::work_experience::{ WorkExperience, WorkExperienceRequest, WorkExperienceResponse};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResponse {
  pub success: bool,
  pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkExperienceListResponse {
    #[serde(rename = "workExperience")]
    pub work_experience: Vec<WorkExperienceResponse>,
}

pub async fn save_work_experience(
    data: Json<WorkExperienceRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<WorkExperienceListResponse>> {
    let person_id = local_user_view.person.id;

    // Use the new replacement strategy - any records not in the request will be deleted
    let work_experience_responses = WorkExperience::save_work_experience_list(
        &mut context.pool(),
        person_id,
        &data.work_experiences,
    ).await?;

    Ok(Json(WorkExperienceListResponse {
        work_experience: work_experience_responses,
    }))
}

pub async fn list_work_experience(
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<WorkExperienceListResponse>> {
    let person_id = local_user_view.person.id;
    let experiences = WorkExperience::read_by_person_id(&mut context.pool(), person_id).await.unwrap_or_else(|_| Vec::new());
    let work_experience_responses: Vec<WorkExperienceResponse> = experiences.into_iter().map(WorkExperienceResponse::from).collect();
    Ok(Json(WorkExperienceListResponse {
        work_experience: work_experience_responses,
    }))
}
