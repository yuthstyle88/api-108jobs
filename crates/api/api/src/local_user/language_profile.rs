use actix_web::web::{Data, Json};
use chrono::Utc;
use lemmy_api_common::account::DeleteItemRequest;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::LanguageProfileId;
use lemmy_db_schema::source::language_profile::{LanguageProfile, LanguageProfileItem, LanguageProfileResponse, LanguageProfileUpdateForm, ListLanguageProfilesResponse, SaveLanguageProfiles};
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;
use serde::{Deserialize, Serialize};
use lemmy_db_schema_file::enums::LanguageLevel;

pub async fn save_language_profiles(
  data: Json<SaveLanguageProfiles>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListLanguageProfilesResponse>> {
  let person_id = local_user_view.person.id;

  // Convert LanguageProfileRequest to LanguageProfileItem
  let language_profile_items: Vec<LanguageProfileItem> = data.language_profiles.iter().map(|lp| {
    LanguageProfileItem {
      id: lp.id,
      lang: Some(lp.lang.clone()),
      level_name: Some(lp.level_name.clone()),
      created_at: Default::default(),
      updated_at: None,
    }
  }).collect();

  // Use the database implementation that handles duplicate validation
  let language_profiles = LanguageProfile::save_language_profile_list(
    &mut context.pool(),
    person_id,
    &language_profile_items,
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
pub struct UpdateLanguageProfileRequest {
  pub id: LanguageProfileId,
  pub lang: String,
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
  _local_user_view: LocalUserView,
) -> FastJobResult<Json<String>> {
  let id = data.into_inner().id;
  LanguageProfile::delete(&mut context.pool(), id).await?;
  Ok(Json("Language profile deleted successfully".to_string()))
}