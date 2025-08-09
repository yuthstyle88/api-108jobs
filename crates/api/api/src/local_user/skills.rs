use actix_web::web::{Data, Json};
use lemmy_api_common::account::{SkillsRequest, UpdateSkillRequest, DeleteItemRequest};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::SkillId;
use lemmy_db_schema::source::skills::{Skills, SkillsInsertForm, SkillsUpdateForm};
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobResult, FastJobErrorType};

// Helper function to validate skill level
fn validate_skill_level(level_id: &Option<i32>) -> FastJobResult<()> {
    if let Some(level) = level_id {
        if *level < 1 || *level > 5 {
            Err(FastJobErrorType::InvalidField("Proficient level must from 1 to 5".to_string()))?;
        }
    }
    Ok(())
}

pub async fn save_skills(
    data: Json<SkillsRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<Vec<Skills>>> {
    let person_id = local_user_view.person.id;

    let mut saved_skills = Vec::new();
    for skill in &data.skills {
        // Validate skill level
        validate_skill_level(&skill.level_id)?;
        
        let saved = match skill.id {
            // Update existing skill record
            Some(id) => {
                let form = SkillsUpdateForm {
                    skill_name: Some(skill.skill_name.clone()),
                    level_id: Some(skill.level_id),
                };

                Skills::update(
                    &mut context.pool(), 
                    id,
                    &form
                ).await?
            }
            // Create new skill record
            None => {
                let form = SkillsInsertForm::new(
                    person_id,
                    skill.skill_name.clone(),
                    skill.level_id,
                );
                Skills::create(&mut context.pool(), &form).await?
            }
        };
        saved_skills.push(saved);
    }

    Ok(Json(saved_skills))
}

pub async fn list_skills(
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<Vec<Skills>>> {
    let person_id = local_user_view.person.id;
    let skills = Skills::read_by_person_id(&mut context.pool(), person_id).await?;
    Ok(Json(skills))
}

pub async fn delete_skills(
    data: Json<Vec<SkillId>>,
    context: Data<FastJobContext>,
) -> FastJobResult<Json<String>> {

    // Delete specific skills records
    for skill_id in data.iter() {
        Skills::delete(&mut context.pool(), *skill_id).await?;
    }
    Ok(Json("Skills records deleted successfully".to_string()))
}

pub async fn update_skill(
    data: Json<UpdateSkillRequest>,
    context: Data<FastJobContext>,
) -> FastJobResult<Json<Skills>> {
    // Validate skill level
    validate_skill_level(&data.level_id)?;
    let form = SkillsUpdateForm{
        skill_name: data.skill_name.clone(),
        level_id: Some(data.level_id),
    };

    let updated_skill = Skills::update(
        &mut context.pool(), 
        data.id, 
        &form
    ).await?;

    Ok(Json(updated_skill))
}

pub async fn delete_single_skill(
    data: Json<DeleteItemRequest<SkillId>>,
    context: Data<FastJobContext>,
) -> FastJobResult<Json<String>> {
    let id = data.into_inner().id;
    
    Skills::delete(&mut context.pool(), id).await?;

    Ok(Json("Skill record deleted successfully".to_string()))
}