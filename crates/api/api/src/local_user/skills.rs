use actix_web::web::{Data, Json};
use lemmy_api_common::account::{DeleteItemRequest};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::SkillId;
use lemmy_db_schema::source::skills::{Skills, SkillsInsertForm, SkillsRequest, SkillsUpdateForm, UpdateSkillRequest, SkillResponse, DeleteSkillsRequest};
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
pub struct SkillsListResponse {
    pub skills: Vec<SkillResponse>,
}

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
) -> FastJobResult<Json<SkillsListResponse>> {
    let person_id = local_user_view.person.id;

    let mut saved_skills = Vec::new();
    for skill in &data.skills {
        match (skill.id, skill.deleted) {
            // Delete existing skill
            (Some(id), true) => {
                Skills::delete(&mut context.pool(), id).await?;
                // Don't add to saved_skills since it's deleted
            }
            // Update existing skill
            (Some(id), false) => {
                // Validate skill level
                validate_skill_level(&skill.level_id)?;
                
                let form = SkillsUpdateForm {
                    skill_name: skill.skill_name.clone(),
                    level_id: Some(skill.level_id),
                };
                let updated = Skills::update(&mut context.pool(), id, &form).await?;
                saved_skills.push(updated);
            }
            // Create new skill
            (None, false) => {
                if let (Some(skill_name), Some(level_id)) = (&skill.skill_name, skill.level_id) {
                    // Validate skill level
                    validate_skill_level(&Some(level_id))?;
                    
                    let form = SkillsInsertForm::new(
                        person_id,
                        skill_name.clone(),
                        Some(level_id),
                    );
                    let created = Skills::create(&mut context.pool(), &form).await?;
                    saved_skills.push(created);
                }
            }
            // Invalid: trying to delete without ID
            (None, true) => {
                // Skip invalid delete requests
            }
        }
    }

    let skill_responses: Vec<SkillResponse> = saved_skills.into_iter().map(SkillResponse::from).collect();
    Ok(Json(SkillsListResponse {
        skills: skill_responses,
    }))
}

pub async fn list_skills(
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<SkillsListResponse>> {
    let person_id = local_user_view.person.id;
    let skills = Skills::read_by_person_id(&mut context.pool(), person_id).await.unwrap_or_else(|_| Vec::new());
    let skill_responses: Vec<SkillResponse> = skills.into_iter().map(SkillResponse::from).collect();
    Ok(Json(SkillsListResponse {
        skills: skill_responses,
    }))
}

pub async fn delete_skills(
    data: Json<DeleteSkillsRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<DeleteResponse>> {
    let person_id = local_user_view.person.id;
    let mut deleted_count = 0;

    for skill_id in data.skill_ids.clone() {
        // First verify the skill belongs to the user
        if let Ok(skill) = Skills::read(&mut context.pool(), skill_id).await {
            if skill.person_id == person_id {
                Skills::delete(&mut context.pool(), skill_id).await?;
                deleted_count += 1;
            }
        }
    }

    Ok(Json(DeleteResponse {
        success: true,
        message: format!("{} records deleted successfully", deleted_count),
    }))
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
) -> FastJobResult<Json<DeleteResponse>> {
    let id = data.into_inner().id;
    
    Skills::delete(&mut context.pool(), id).await?;

    Ok(Json(DeleteResponse {
        success: true,
        message: "1 record deleted successfully".to_string(),
    }))
}