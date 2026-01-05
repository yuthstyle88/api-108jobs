use actix_web::web::Data;
use actix_web::web::Json;
use diesel_async::scoped_futures::ScopedFutureExt;
use app_108jobs_api_utils::{context::FastJobContext, utils::is_admin};
use app_108jobs_db_schema::{
  source::{
    local_user::{LocalUser, LocalUserUpdateForm},
    registration_application::{RegistrationApplication, RegistrationApplicationUpdateForm},
  },
  traits::Crud,
  utils::{diesel_string_update, get_conn},
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_registration_applications::{
  api::{ApproveRegistrationApplication, RegistrationApplicationResponse},
  RegistrationApplicationView,
};
use app_108jobs_utils::error::FastJobResult;

pub async fn approve_registration_application(
  data: Json<ApproveRegistrationApplication>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<RegistrationApplicationResponse>> {
  let app_id = data.id;

  // Only let admins do this
  is_admin(&local_user_view)?;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  let tx_data = data.clone();
  conn
      .run_transaction(|conn| {
        async move {
          // Update the registration with reason, admin_id
          let deny_reason = diesel_string_update(tx_data.deny_reason.as_deref());
          let app_form = RegistrationApplicationUpdateForm {
            admin_id: Some(Some(local_user_view.person.id)),
            deny_reason,
          };

          let registration_application =
              RegistrationApplication::update(&mut conn.into(), app_id, &app_form).await?;

          // Update the local_user row
          let local_user_form = LocalUserUpdateForm {
            accepted_application: Some(tx_data.approve),
            ..Default::default()
          };

          let approved_user_id = registration_application.local_user_id;
          LocalUser::update(&mut conn.into(), approved_user_id, &local_user_form).await?;

          Ok(approved_user_id)
        }
            .scope_boxed()
      })
      .await?;

  // Read the view
  let registration_application =
    RegistrationApplicationView::read(&mut context.pool(), app_id).await?;

  Ok(Json(RegistrationApplicationResponse {
    registration_application,
  }))
}
