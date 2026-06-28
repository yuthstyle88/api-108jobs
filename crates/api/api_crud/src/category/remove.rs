use actix_web::web::{Data, Json};
use app_108jobs_api_utils::{
  build_response::build_category_response,
  context::FastJobContext,
  utils::{check_category_deleted_removed, is_admin},
};
use app_108jobs_core::error::FastJobResult;
use app_108jobs_db::{
  source::category::{Category, CategoryUpdateForm},
  traits::Crud,
};
use app_108jobs_db_views_category::api::{CategoryResponse, RemoveCategory};
use app_108jobs_db_views_local_user::LocalUserView;

pub async fn remove_category(
  data: Json<RemoveCategory>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CategoryResponse>> {
  let category = Category::read(&mut context.pool(), data.category_id).await?;
  check_category_deleted_removed(&category)?;

  // Verify its an admin (only an admin can remove a category)
  is_admin(&local_user_view)?;

  // Do the remove
  let category_id = data.category_id;
  let removed = data.removed;
  Category::update(
    &mut context.pool(),
    category_id,
    &CategoryUpdateForm {
      removed: Some(removed),
      ..Default::default()
    },
  )
  .await?;

  build_category_response(&context, local_user_view, category_id).await
}
