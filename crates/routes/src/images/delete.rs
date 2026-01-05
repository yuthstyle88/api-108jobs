use super::utils::delete_old_image;
use actix_web::web::*;
use app_108jobs_api_utils::{
  context::FastJobContext,
  request::{delete_image_alias, purge_image_from_pictrs},
  utils::is_admin,
};
use app_108jobs_db_schema::{
  source::{
    category::{Category, CategoryUpdateForm},
    images::LocalImage,
    person::{Person, PersonUpdateForm},
    site::{Site, SiteUpdateForm},
  },
  traits::Crud,
};
use app_108jobs_db_views_category::api::CategoryIdQuery;
use app_108jobs_db_views_local_image::api::DeleteImageParams;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_utils::error::FastJobResult;

pub async fn delete_site_icon(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let site = Site::read_local(&mut context.pool()).await?;
  is_admin(&local_user_view)?;

  delete_old_image(&site.icon, &context).await?;

  let form = SiteUpdateForm {
    icon: Some(None),
    ..Default::default()
  };
  Site::update(&mut context.pool(), site.id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}
pub async fn delete_site_banner(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let site = Site::read_local(&mut context.pool()).await?;
  is_admin(&local_user_view)?;

  delete_old_image(&site.banner, &context).await?;

  let form = SiteUpdateForm {
    banner: Some(None),
    ..Default::default()
  };
  Site::update(&mut context.pool(), site.id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}

pub async fn delete_category_icon(
  data: Json<CategoryIdQuery>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  is_admin(&local_user_view)?;
  
  let category = Category::read(&mut context.pool(), data.id).await?;
  delete_old_image(&category.icon, &context).await?;

  let form = CategoryUpdateForm {
    icon: Some(None),
    ..Default::default()
  };
  Category::update(&mut context.pool(), category.id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}

pub async fn delete_category_banner(
  data: Json<CategoryIdQuery>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  is_admin(&local_user_view)?;
  
  let category = Category::read(&mut context.pool(), data.id).await?;
  delete_old_image(&category.icon, &context).await?;

  let form = CategoryUpdateForm {
    icon: Some(None),
    ..Default::default()
  };
  Category::update(&mut context.pool(), category.id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}

pub async fn delete_user_avatar(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  delete_old_image(&local_user_view.person.avatar, &context).await?;

  let form = PersonUpdateForm {
    avatar: Some(None),
    ..Default::default()
  };
  Person::update(&mut context.pool(), local_user_view.person.id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}

pub async fn delete_user_banner(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  delete_old_image(&local_user_view.person.banner, &context).await?;

  let form = PersonUpdateForm {
    banner: Some(None),
    ..Default::default()
  };
  Person::update(&mut context.pool(), local_user_view.person.id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}

/// Deletes an image for a specific user.
pub async fn delete_image(
  data: Json<DeleteImageParams>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  LocalImage::validate_by_alias_and_user(
    &mut context.pool(),
    &data.filename,
    local_user_view.person.id,
  )
  .await?;

  delete_image_alias(&data.filename, &context).await?;

  Ok(Json(SuccessResponse::default()))
}

/// Deletes any image, only for admins.
pub async fn delete_image_admin(
  data: Json<DeleteImageParams>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  is_admin(&local_user_view)?;

  // Use purge, since it should remove any other aliases.
  purge_image_from_pictrs(&data.filename, &context).await?;

  Ok(Json(SuccessResponse::default()))
}
