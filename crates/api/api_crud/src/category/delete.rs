use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::utils::{check_category_deleted_removed, is_admin};
use app_108jobs_api_utils::{
  build_response::build_category_response,
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use app_108jobs_db_schema::{
  source::category::{Category, CategoryUpdateForm},
  traits::Crud,
};
use app_108jobs_db_views_category::api::{CategoryResponse, DeleteCategory};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_utils::error::FastJobResult;

pub async fn delete_category(
  data: Json<DeleteCategory>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CategoryResponse>> {
  is_admin(&local_user_view)?;

  let category = Category::read(&mut context.pool(), data.category_id).await?;
  check_category_deleted_removed(&category)?;

  // Do the delete
  let category_id = data.category_id;
  let deleted = data.deleted;
  let category = Category::update(
    &mut context.pool(),
    category_id,
    &CategoryUpdateForm {
      deleted: Some(deleted),
      ..Default::default()
    },
  )
  .await?;

  ActivityChannel::submit_activity(
    SendActivityData::DeleteCategory(local_user_view.person.clone(), category, data.deleted),
    &context,
  )?;

  build_category_response(&context, local_user_view, category_id).await
}
