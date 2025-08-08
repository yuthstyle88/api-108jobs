use actix_web::web::{Data, Json};
use lemmy_api_common::account::{EducationRequest, UpdateEducationRequest, DeleteItemRequest};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::education::{Education, EducationInsertForm};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub async fn save_education(
    data: Json<EducationRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<Vec<Education>>> {
    let person_id = local_user_view.person.id;

    let mut saved_educations = Vec::new();
    for edu in &data.educations {
        let saved = match edu.id {
            // Update existing education record
            Some(id) => {
                Education::update_by_id_and_person(
                    &mut context.pool(), 
                    id, 
                    person_id, 
                    edu.school_name.clone(), 
                    edu.major.clone()
                ).await?
            }
            // Create new education record
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

    Ok(Json(saved_educations))
}

pub async fn list_education(
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<Vec<Education>>> {
    let person_id = local_user_view.person.id;
    let educations = Education::read_by_person_id(&mut context.pool(), person_id).await?;
    Ok(Json(educations))
}

pub async fn delete_education(
    data: Json<Vec<i32>>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<String>> {
    let person_id = local_user_view.person.id;
    
    // Delete specific education records
    for education_id in data.iter() {
        Education::delete_by_id_and_person(&mut context.pool(), *education_id, person_id).await?;
    }

    Ok(Json("Education records deleted successfully".to_string()))
}

pub async fn update_education(
    data: Json<UpdateEducationRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<Education>> {
    let person_id = local_user_view.person.id;
    
    let updated_education = Education::update_by_id_and_person(
        &mut context.pool(), 
        data.id, 
        person_id, 
        data.school_name.clone(), 
        data.major.clone()
    ).await?;

    Ok(Json(updated_education))
}

pub async fn delete_single_education(
    data: Json<DeleteItemRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<String>> {
    let person_id = local_user_view.person.id;
    
    Education::delete_by_id_and_person(&mut context.pool(), data.id, person_id).await?;

    Ok(Json("Education record deleted successfully".to_string()))
}