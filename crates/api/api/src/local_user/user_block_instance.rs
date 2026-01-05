use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::{
  source::instance::{InstanceActions, InstanceBlockForm},
  traits::Blockable,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::{SuccessResponse, UserBlockInstanceParams};
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};

pub async fn user_block_instance(
  data: Json<UserBlockInstanceParams>,
  local_user_view: LocalUserView,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<SuccessResponse>> {
  let instance_id = data.instance_id;
  let person_id = local_user_view.person.id;
  if local_user_view.person.instance_id == instance_id {
    return Err(FastJobErrorType::CantBlockLocalInstance)?;
  }

  let instance_block_form = InstanceBlockForm::new(person_id, instance_id);

  if data.block {
    InstanceActions::block(&mut context.pool(), &instance_block_form).await?;
  } else {
    InstanceActions::unblock(&mut context.pool(), &instance_block_form).await?;
  }

  Ok(Json(SuccessResponse::default()))
}
