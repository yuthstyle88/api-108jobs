use actix_web::web::{Data, Json};
use lemmy_api_common::account::{SkillsRequest, UpdateSkillRequest, DeleteItemRequest};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::skills::{Skills, SkillsInsertForm};
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
                Skills::update_by_id_and_person(
                    &mut context.pool(), 
                    id, 
                    person_id, 
                    skill.skill_name.clone(),
                    skill.level_id,
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
    data: Json<Vec<i32>>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<String>> {
    let person_id = local_user_view.person.id;
    
    // Delete specific skills records
    for skill_id in data.iter() {
        Skills::delete_by_id_and_person(&mut context.pool(), *skill_id, person_id).await?;
    }

    Ok(Json("Skills records deleted successfully".to_string()))
}

pub async fn update_skill(
    data: Json<UpdateSkillRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<Skills>> {
    let person_id = local_user_view.person.id;
    
    // Validate skill level
    validate_skill_level(&data.level_id)?;
    
    let updated_skill = Skills::update_by_id_and_person(
        &mut context.pool(), 
        data.id, 
        person_id, 
        data.skill_name.clone(),
        data.level_id,
    ).await?;

    Ok(Json(updated_skill))
}

pub async fn delete_single_skill(
    data: Json<DeleteItemRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<String>> {
    let person_id = local_user_view.person.id;
    
    Skills::delete_by_id_and_person(&mut context.pool(), data.id, person_id).await?;

    Ok(Json("Skill record deleted successfully".to_string()))
}