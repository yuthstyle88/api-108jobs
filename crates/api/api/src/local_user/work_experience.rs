use actix_web::web::{Data, Json};
use lemmy_api_common::account::{WorkExperienceRequest, UpdateWorkExperienceRequest, DeleteItemRequest};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::work_experience::{WorkExperience, WorkExperienceInsertForm};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub async fn save_work_experience(
    data: Json<WorkExperienceRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<Vec<WorkExperience>>> {
    let person_id = local_user_view.person.id;

    let mut saved_experiences = Vec::new();
    for exp in &data.work_experiences {
        let saved = match exp.id {
            // Update existing work experience record
            Some(id) => {
                WorkExperience::update_by_id_and_person(
                    &mut context.pool(), 
                    id, 
                    person_id, 
                    exp.company_name.clone(),
                    exp.position.clone(),
                    exp.start_month.clone(),
                    exp.start_year,
                    exp.end_month.clone(),
                    exp.end_year,
                    exp.is_current,
                ).await?
            }
            // Create new work experience record
            None => {
                let form = WorkExperienceInsertForm::new(
                    person_id,
                    exp.company_name.clone(),
                    exp.position.clone(),
                    exp.start_month.clone(),
                    exp.start_year,
                    exp.end_month.clone(),
                    exp.end_year,
                );
                WorkExperience::create(&mut context.pool(), &form).await?
            }
        };
        saved_experiences.push(saved);
    }

    Ok(Json(saved_experiences))
}

pub async fn list_work_experience(
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<Vec<WorkExperience>>> {
    let person_id = local_user_view.person.id;
    let experiences = WorkExperience::read_by_person_id(&mut context.pool(), person_id).await?;
    Ok(Json(experiences))
}

pub async fn delete_work_experience(
    data: Json<Vec<i32>>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<String>> {
    let person_id = local_user_view.person.id;
    
    // Delete specific work experience records
    for experience_id in data.iter() {
        WorkExperience::delete_by_id_and_person(&mut context.pool(), *experience_id, person_id).await?;
    }

    Ok(Json("Work experience records deleted successfully".to_string()))
}

pub async fn update_work_experience(
    data: Json<UpdateWorkExperienceRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<WorkExperience>> {
    let person_id = local_user_view.person.id;
    
    let updated_experience = WorkExperience::update_by_id_and_person(
        &mut context.pool(), 
        data.id, 
        person_id, 
        data.company_name.clone(),
        data.position.clone(),
        data.start_month.clone(),
        data.start_year,
        data.end_month.clone(),
        data.end_year,
        data.is_current,
    ).await?;

    Ok(Json(updated_experience))
}

pub async fn delete_single_work_experience(
    data: Json<DeleteItemRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<String>> {
    let person_id = local_user_view.person.id;
    
    WorkExperience::delete_by_id_and_person(&mut context.pool(), data.id, person_id).await?;

    Ok(Json("Work experience record deleted successfully".to_string()))
}