use crate::site::registration_applications::{
  approve::approve_registration_application,
  list::list_registration_applications,
  unread_count::get_unread_registration_application_count,
};
use actix_web::web::{Data, Json, Query};
use lemmy_api_crud::site::update::update_site;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::{
  newtypes::InstanceId,
  source::{
    local_site::{LocalSite, LocalSiteUpdateForm},
    local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
    person::{Person, PersonInsertForm},
    registration_application::{RegistrationApplication, RegistrationApplicationInsertForm},
  },
  test_data::TestData,
  traits::Crud,
  utils::DbPool,
};
use lemmy_db_schema_file::enums::RegistrationMode;
use lemmy_db_views_inbox_combined::api::GetUnreadRegistrationApplicationCountResponse;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_registration_applications::api::{
  ApproveRegistrationApplication,
  ListRegistrationApplicationsResponse,
};
use lemmy_db_views_site::api::EditSite;
use lemmy_utils::{
  error::{FastJobErrorType, FastJobResult},
  CACHE_DURATION_API,
};
use serial_test::serial;

async fn create_test_site(context: &Data<FastJobContext>) -> FastJobResult<(TestData, LocalUserView)> {
  let pool = &mut context.pool();
  let data = TestData::create(pool).await?;

  // Enable some local site settings
  let local_site_form = LocalSiteUpdateForm {
    require_email_verification: Some(true),
    application_question: Some(Some(".".to_string())),
    registration_mode: Some(RegistrationMode::RequireApplication),
    site_setup: Some(true),
    ..Default::default()
  };
  LocalSite::update(pool, &local_site_form).await?;

  let admin_person = Person::create(
    pool,
    &PersonInsertForm::test_form(data.instance.id, "admin"),
  )
  .await?;
  LocalUser::create(
    pool,
    &LocalUserInsertForm::test_form_admin(admin_person.id),
    vec![],
  )
  .await?;

  let admin_local_user_view = LocalUserView::read_person(pool, admin_person.id).await?;

  Ok((data, admin_local_user_view))
}

async fn signup(
  pool: &mut DbPool<'_>,
  instance_id: InstanceId,
  name: &str,
  email: Option<&str>,
) -> FastJobResult<(LocalUser, RegistrationApplication)> {
  let person_insert_form = PersonInsertForm::test_form(instance_id, name);
  let person = Person::create(pool, &person_insert_form).await?;

  let local_user_insert_form = match email {
    Some(email) => LocalUserInsertForm {
      email: Some(email.to_string()),
      email_verified: Some(false),
      ..LocalUserInsertForm::test_form(person.id)
    },
    None => LocalUserInsertForm::test_form(person.id),
  };

  let local_user = LocalUser::create(pool, &local_user_insert_form, vec![]).await?;

  let application_insert_form = RegistrationApplicationInsertForm {
    local_user_id: local_user.id,
    answer: "x".to_string(),
  };
  let application = RegistrationApplication::create(pool, &application_insert_form).await?;

  Ok((local_user, application))
}

async fn get_application_statuses(
  context: &Data<FastJobContext>,
  admin: LocalUserView,
) -> FastJobResult<(
  Json<GetUnreadRegistrationApplicationCountResponse>,
  Json<ListRegistrationApplicationsResponse>,
  Json<ListRegistrationApplicationsResponse>,
)> {
  let application_count =
    get_unread_registration_application_count(context.clone(), admin.clone()).await?;

  let unread_applications = list_registration_applications(
    Query::from_query("unread_only=true")?,
    context.clone(),
    admin.clone(),
  )
  .await?;

  let all_applications = list_registration_applications(
    Query::from_query("unread_only=false")?,
    context.clone(),
    admin,
  )
  .await?;

  Ok((application_count, unread_applications, all_applications))
}
