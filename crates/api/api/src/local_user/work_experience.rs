use actix_web::web::{Data, Json};
use lemmy_api_common::account::{DeleteItemRequest};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::WorkExperienceId;
use lemmy_db_schema::source::work_experience::{UpdateWorkExperienceRequest, WorkExperience, WorkExperienceInsertForm, WorkExperienceRequest, WorkExperienceUpdateForm};
use lemmy_db_schema::traits::Crud;
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
                let form = WorkExperienceUpdateForm {
                    company_name: Some(exp.company_name.clone()),
                    position: Some(exp.position.clone()),
                    start_date: Some(exp.start_date),
                    end_date: Some(exp.end_date),
                    is_current: Some(exp.is_current),
                };
                WorkExperience::update(
                    &mut context.pool(), 
                    id, 
                   &form
                ).await?
            }
            // Create new work experience record
            None => {
                let form = WorkExperienceInsertForm::new(
                    person_id,
                    exp.company_name.clone(),
                    exp.position.clone(),
                    exp.start_date,
                    exp.end_date,
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
    data: Json<Vec<WorkExperienceId>>,
    context: Data<FastJobContext>,
) -> FastJobResult<Json<String>> {

    // Delete specific work experience records
    for experience_id in data.iter() {
        WorkExperience::delete(&mut context.pool(), *experience_id).await?;
    }

    Ok(Json("Work experience records deleted successfully".to_string()))
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
) -> FastJobResult<Json<String>> {
    let id: WorkExperienceId = data.into_inner().id;
    WorkExperience::delete(&mut context.pool(), id).await?;

    Ok(Json("Work experience record deleted successfully".to_string()))
}