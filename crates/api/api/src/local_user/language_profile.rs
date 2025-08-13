use actix_web::web::{Data, Json};
use lemmy_api_common::account::DeleteItemRequest;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::LanguageProfileId;
use lemmy_db_schema::source::language_profile::{
  LanguageProfile, LanguageProfileInsertForm, LanguageProfileResponse,
  SaveLanguageProfiles, ListLanguageProfilesResponse, LanguageProfileUpdateForm, LanguageLevel,
  DeleteLanguageProfilesRequest,
};
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobResult, FastJobErrorType};
use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResponse {
  pub success: bool,
  pub message: String,
}

pub async fn save_language_profiles(
  data: Json<SaveLanguageProfiles>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListLanguageProfilesResponse>> {
  let person_id = local_user_view.person.id;

  // Use the database implementation that handles duplicate validation
  let language_profiles = LanguageProfile::save_language_profiles(
    &mut context.pool(),
    person_id,
    data.language_profiles.clone(),
  ).await?;

  Ok(Json(ListLanguageProfilesResponse {
    language_profiles,
  }))
}

pub async fn list_language_profiles(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListLanguageProfilesResponse>> {
  let person_id = local_user_view.person.id;

  let language_profiles = LanguageProfile::list_for_person(&mut context.pool(), person_id).await.unwrap_or_else(|_| Vec::new());

  Ok(Json(ListLanguageProfilesResponse {
    language_profiles,
  }))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLanguageProfileRequest {
  pub id: LanguageProfileId,
  pub lang: String,
  #[serde(rename = "level")]
  pub level_name: LanguageLevel,
}


pub async fn update_language_profile(
  data: Json<UpdateLanguageProfileRequest>,
  context: Data<FastJobContext>,
  _local_user_view: LocalUserView,
) -> FastJobResult<Json<LanguageProfileResponse>> {
  let update_form = LanguageProfileUpdateForm {
    lang: Some(data.lang.clone()),
    level_name: Some(data.level_name.clone()),
    updated_at: Some(Utc::now()),
  };

  let updated_profile = LanguageProfile::update(&mut context.pool(), data.id, &update_form).await?;

  Ok(Json(LanguageProfileResponse::from(updated_profile)))
}

pub async fn delete_single_language_profile(
  data: Json<DeleteItemRequest<LanguageProfileId>>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<DeleteResponse>> {
  let id = data.into_inner().id;
  let person_id = local_user_view.person.id;
  
  // First verify the profile belongs to the user
  if let Ok(profile) = LanguageProfile::read(&mut context.pool(), id).await {
    if profile.person_id == person_id {
      LanguageProfile::delete(&mut context.pool(), id).await?;
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

pub async fn delete_language_profiles(
  data: Json<DeleteLanguageProfilesRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<DeleteResponse>> {
  let person_id = local_user_view.person.id;
  let mut deleted_count = 0;

  for profile_id in data.language_profile_ids.clone() {
    // First verify the profile belongs to the user
    if let Ok(profile) = LanguageProfile::read(&mut context.pool(), profile_id).await {
      if profile.person_id == person_id {
        LanguageProfile::delete(&mut context.pool(), profile_id).await?;
        deleted_count += 1;
      }
    }
  }

  Ok(Json(DeleteResponse {
    success: true,
    message: format!("{} records deleted successfully", deleted_count),
  }))
}