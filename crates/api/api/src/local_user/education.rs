use actix_web::web::{Data, Json};
use lemmy_api_common::account::{DeleteItemRequest};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::EducationId;
use lemmy_db_schema::source::education::{Education, EducationRequest, EducationUpdateForm, UpdateEducationRequest, EducationResponse, DeleteEducationsRequest};
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
pub struct EducationListResponse {
    pub education: Vec<EducationResponse>,
}

pub async fn save_education(
    data: Json<EducationRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<EducationListResponse>> {
    let person_id = local_user_view.person.id;

    // Use the new replacement strategy - any records not in the request will be deleted
    let education_responses = Education::save_education_list(
        &mut context.pool(),
        person_id,
        &data.education,
    ).await?;

    Ok(Json(EducationListResponse {
        education: education_responses,
    }))
}

pub async fn list_education(
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<EducationListResponse>> {
    let person_id = local_user_view.person.id;
    let educations = Education::read_by_person_id(&mut context.pool(), person_id).await.unwrap_or_else(|_| Vec::new());
    let education_responses: Vec<EducationResponse> = educations.into_iter().map(EducationResponse::from).collect();
    Ok(Json(EducationListResponse {
        education: education_responses,
    }))
}

pub async fn delete_educations(
    data: Json<DeleteEducationsRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<DeleteResponse>> {
    let person_id = local_user_view.person.id;
    let mut deleted_count = 0;

    for education_id in data.education_ids.clone() {
        // First verify the education belongs to the user
        if let Ok(education) = Education::read(&mut context.pool(), education_id).await {
            if education.person_id == person_id {
                Education::delete(&mut context.pool(), education_id).await?;
                deleted_count += 1;
            }
        }
    }

    Ok(Json(DeleteResponse {
        success: true,
        message: format!("{} records deleted successfully", deleted_count),
    }))
}

pub async fn update_education(
    data: Json<UpdateEducationRequest>,
    context: Data<FastJobContext>,
) -> FastJobResult<Json<Education>> {

    let form = EducationUpdateForm{
        school_name:  data.school_name.clone(),
        major: data.major.clone()
    };
    let updated_education = Education::update(
        &mut context.pool(), 
        data.id,
        &form
    ).await?;

    Ok(Json(updated_education))
}

pub async fn delete_single_education(
    data: Json<DeleteItemRequest<EducationId>>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<DeleteResponse>> {
    let id: EducationId = data.into_inner().id;
    let person_id = local_user_view.person.id;
    
    // First verify the education belongs to the user
    if let Ok(education) = Education::read(&mut context.pool(), id).await {
        if education.person_id == person_id {
            Education::delete(&mut context.pool(), id).await?;
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