use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{context::FastJobContext, utils::is_admin};
use lemmy_db_schema::{
  source::{
    instance::Instance,
    mod_log::admin::{AdminAllowInstance, AdminAllowInstanceForm},
  },
  traits::Crud,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::{AdminAllowInstanceParams, SuccessResponse};
use lemmy_utils::error::FastJobResult;

pub async fn admin_allow_instance(
  data: Json<AdminAllowInstanceParams>,
  local_user_view: LocalUserView,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<SuccessResponse>> {
  is_admin(&local_user_view)?;

  let instance_id = Instance::read_or_create(&mut context.pool(), data.instance.clone())
    .await?
    .id;

  let mod_log_form = AdminAllowInstanceForm {
    instance_id,
    admin_person_id: local_user_view.person.id,
    reason: data.reason.clone(),
    allowed: data.allow,
  };
  AdminAllowInstance::create(&mut context.pool(), &mod_log_form).await?;

  Ok(Json(SuccessResponse::default()))
}
