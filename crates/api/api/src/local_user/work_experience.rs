use actix_web::web::{Data, Json};
use lemmy_api_common::account::{DeleteItemRequest};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::WorkExperienceId;
use lemmy_db_schema::source::work_experience::{UpdateWorkExperienceRequest, WorkExperience, WorkExperienceInsertForm, WorkExperienceRequest, WorkExperienceUpdateForm, DeleteWorkExperiencesRequest, WorkExperienceResponse};
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobResult, FastJobErrorType};
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

    let mut saved_experiences = Vec::new();
    for exp in &data.work_experiences {
        match (exp.id, exp.deleted) {
            // Delete existing work experience
            (Some(id), true) => {
                WorkExperience::delete(&mut context.pool(), id).await?;
                // Don't add to saved_experiences since it's deleted
            }
            // Update existing work experience
            (Some(id), false) => {
                let form = WorkExperienceUpdateForm {
                    company_name: exp.company_name.clone(),
                    position: exp.position.clone(),
                    start_date: exp.start_date,
                    end_date: Some(exp.end_date),
                    is_current: Some(exp.is_current),
                };
                let updated = WorkExperience::update(&mut context.pool(), id, &form).await?;
                saved_experiences.push(updated);
            }
            // Create new work experience
            (None, false) => {
                if let (Some(company_name), Some(position), Some(start_date)) = (&exp.company_name, &exp.position, exp.start_date) {
                    let form = WorkExperienceInsertForm::new(
                        person_id,
                        company_name.clone(),
                        position.clone(),
                        start_date,
                        exp.end_date,
                    );
                    let created = WorkExperience::create(&mut context.pool(), &form).await?;
                    saved_experiences.push(created);
                }
            }
            // Invalid: trying to delete without ID
            (None, true) => {
                // Skip invalid delete requests
            }
        }
    }

    let work_experience_responses: Vec<WorkExperienceResponse> = saved_experiences.into_iter().map(WorkExperienceResponse::from).collect();
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

pub async fn delete_work_experience(
    data: Json<DeleteWorkExperiencesRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<DeleteResponse>> {
    let person_id = local_user_view.person.id;
    let mut deleted_count = 0;

    for experience_id in data.work_experience_ids.clone() {
        // First verify the work experience belongs to the user
        if let Ok(experience) = WorkExperience::read(&mut context.pool(), experience_id).await {
            if experience.person_id == person_id {
                WorkExperience::delete(&mut context.pool(), experience_id).await?;
                deleted_count += 1;
            }
        }
    }

    Ok(Json(DeleteResponse {
        success: true,
        message: format!("{} records deleted successfully", deleted_count),
    }))
}

pub async fn update_work_experience(
    data: Json<UpdateWorkExperienceRequest>,
    context: Data<FastJobContext>,
) -> FastJobResult<Json<WorkExperience>> {
    let form = WorkExperienceUpdateForm {
        company_name: data.company_name.clone(),
        position: data.position.clone(),
        start_date: data.start_date,
        end_date: Some(data.end_date),
        is_current: Some(data.is_current),
    };
    let updated_experience = WorkExperience::update(
        &mut context.pool(), 
        data.id, 
        &form
    ).await?;

    Ok(Json(updated_experience))
}

pub async fn delete_single_work_experience(
    data: Json<DeleteItemRequest<WorkExperienceId>>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<DeleteResponse>> {
    let id: WorkExperienceId = data.into_inner().id;
    let person_id = local_user_view.person.id;
    
    // First verify the work experience belongs to the user
    if let Ok(experience) = WorkExperience::read(&mut context.pool(), id).await {
        if experience.person_id == person_id {
            WorkExperience::delete(&mut context.pool(), id).await?;
            Ok(Json(DeleteResponse {
                success: true,
                message: "1 record deleted successfully".to_string(),
            }))
        } else {
            Err(FastJobErrorType::NotFound)?
        }
    } else {
        Err(FastJobErrorType::NotFound)?
    }
}