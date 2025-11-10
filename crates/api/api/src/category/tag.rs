use actix_web::web::Data;
use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::{check_category_deleted_removed, is_admin};
use lemmy_db_schema::{
  source::{
    category::Category,
    tag::{Tag, TagInsertForm, TagUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views_category::api::{CreateCategoryTag, DeleteCategoryTag, UpdateCategoryTag};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{error::FastJobResult, utils::validation::tag_name_length_check};

pub async fn create_category_tag(
  data: Json<CreateCategoryTag>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<Tag>> {
  is_admin(&local_user_view)?;

  let category = Category::read(&mut context.pool(), data.category_id).await?;

  tag_name_length_check(&data.display_name)?;
  // Verify that only mods can create tags
  check_category_deleted_removed(&category)?;

  // Create the tag
  let tag_form = TagInsertForm {
    display_name: data.display_name.clone(),
    category_id: data.category_id,
  };

  let tag = Tag::create(&mut context.pool(), &tag_form).await?;

  Ok(Json(tag))
}

pub async fn update_category_tag(
  data: Json<UpdateCategoryTag>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<Tag>> {
  is_admin(&local_user_view)?;

  let tag = Tag::read(&mut context.pool(), data.tag_id).await?;
  let category = Category::read(&mut context.pool(), tag.category_id).await?;

  // Verify that only mods can update tags
  check_category_deleted_removed(&category)?;

  tag_name_length_check(&data.display_name)?;
  // Update the tag
  let tag_form = TagUpdateForm {
    display_name: Some(data.display_name.clone()),
    updated_at: Some(Some(Utc::now())),
    ..Default::default()
  };

  let tag = Tag::update(&mut context.pool(), data.tag_id, &tag_form).await?;

  Ok(Json(tag))
}

pub async fn delete_category_tag(
  data: Json<DeleteCategoryTag>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<Tag>> {
  is_admin(&local_user_view)?;

  let tag = Tag::read(&mut context.pool(), data.tag_id).await?;
  let category = Category::read(&mut context.pool(), tag.category_id).await?;

  check_category_deleted_removed(&category)?;

  // Soft delete the tag
  let tag_form = TagUpdateForm {
    updated_at: Some(Some(Utc::now())),
    deleted: Some(true),
    ..Default::default()
  };

  let tag = Tag::update(&mut context.pool(), data.tag_id, &tag_form).await?;

  Ok(Json(tag))
}
