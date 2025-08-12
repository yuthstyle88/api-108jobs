use actix_web::web::{Data, Json};
use lemmy_api_common::account::DeleteItemRequest;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::LanguageProfileId;
use lemmy_db_schema::source::language_profile::{
  LanguageProfile, LanguageProfileInsertForm, LanguageProfileResponse,
  SaveLanguageProfiles, ListLanguageProfilesResponse, LanguageProfileUpdateForm,
};
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;
use serde::{Deserialize, Serialize};
use chrono::Utc;

pub async fn save_language_profiles(
  data: Json<SaveLanguageProfiles>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<Vec<LanguageProfile>>> {
  let person_id = local_user_view.person.id;

  let mut saved_profiles = Vec::new();
  for profile_request in data.language_profiles.clone() {
    let result = match profile_request.id {
      Some(id) => {
        // Update existing profile
        let update_form = LanguageProfileUpdateForm {
          lang: Some(profile_request.lang),
          level_name: Some(profile_request.level_name),
          updated_at: Some(chrono::Utc::now()),
        };
        LanguageProfile::update(&mut context.pool(), id, &update_form).await?
      }
      None => {
        // Create new profile
        let insert_form = LanguageProfileInsertForm::new(
          person_id,
          profile_request.lang,
          profile_request.level_name,
        );
        LanguageProfile::create(&mut context.pool(), &insert_form).await?
      }
    };
    saved_profiles.push(result);
  }

  Ok(Json(saved_profiles))
}

pub async fn list_language_profiles(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListLanguageProfilesResponse>> {
  let person_id = local_user_view.person.id;

  let language_profiles = LanguageProfile::list_for_person(&mut context.pool(), person_id).await?;

  Ok(Json(ListLanguageProfilesResponse {
    language_profiles,
  }))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateLanguageProfileRequest {
  pub id: LanguageProfileId,
  pub lang: String,
  pub level_name: String,
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