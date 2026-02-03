use crate::api::{Register, RegisterRequest};
use crate::RegistrationApplicationView;
use diesel::{
  dsl::count, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use app_108jobs_db_schema::sensitive::SensitiveString;
use app_108jobs_db_schema::utils::get_required_sensitive;
use app_108jobs_db_schema::{
  aliases,
  newtypes::{PaginationCursor, PersonId, RegistrationApplicationId},
  source::registration_application::RegistrationApplication,
  traits::{Crud, PaginationCursorBuilder},
  utils::{get_conn, limit_fetch, paginate, DbPool},
};
use app_108jobs_db_schema_file::schema::{local_user, person, registration_application};
use app_108jobs_utils::error::{FastJobError, FastJobErrorExt, FastJobErrorType, FastJobResult};
use app_108jobs_utils::utils::random::rand_number5;
use app_108jobs_utils::utils::validation::is_valid_email;

impl PaginationCursorBuilder for RegistrationApplicationView {
  type CursorData = RegistrationApplication;
  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new_single('R', self.registration_application.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    let id = cursor.first_id()?;
    RegistrationApplication::read(pool, RegistrationApplicationId(id)).await
  }
}

impl RegistrationApplicationView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    let local_user_join =
      local_user::table.on(registration_application::local_user_id.eq(local_user::id));

    let creator_join = person::table.on(local_user::person_id.eq(person::id));
    let admin_join = aliases::person1
      .on(registration_application::admin_id.eq(aliases::person1.field(person::id).nullable()));

    registration_application::table
      .inner_join(local_user_join)
      .inner_join(creator_join)
      .left_join(admin_join)
  }

  pub async fn read(pool: &mut DbPool<'_>, id: RegistrationApplicationId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(registration_application::id.eq(id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn read_by_person(pool: &mut DbPool<'_>, person_id: PersonId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(person::id.eq(person_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  /// Returns the current unread registration_application count
  pub async fn get_unread_count(
    pool: &mut DbPool<'_>,
    verified_email_only: bool,
  ) -> FastJobResult<i64> {
    let conn = &mut get_conn(pool).await?;

    let mut query = Self::joins()
      .filter(RegistrationApplication::is_unread())
      .select(count(registration_application::id))
      .into_boxed();

    if verified_email_only {
      query = query.filter(local_user::email_verified.eq(true))
    }

    query
      .first::<i64>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}

#[derive(Default)]
pub struct RegistrationApplicationQuery {
  pub unread_only: Option<bool>,
  pub verified_email_only: Option<bool>,
  pub cursor_data: Option<RegistrationApplication>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

impl RegistrationApplicationQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Vec<RegistrationApplicationView>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(self.limit)?;
    let o = self;

    let mut query = RegistrationApplicationView::joins()
      .select(RegistrationApplicationView::as_select())
      .limit(limit)
      .into_boxed();

    if o.unread_only.unwrap_or_default() {
      query = query
        .filter(RegistrationApplication::is_unread())
        .order_by(registration_application::published_at.asc());
    } else {
      query = query.order_by(registration_application::published_at.desc());
    }

    if o.verified_email_only.unwrap_or_default() {
      query = query.filter(local_user::email_verified.eq(true))
    }

    // Sorting by published
    let paginated_query = paginate(query, SortDirection::Desc, o.cursor_data, None, o.page_back);

    paginated_query
      .load::<RegistrationApplicationView>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}

impl TryFrom<RegisterRequest> for Register {
  type Error = FastJobError;

  fn try_from(mut form: RegisterRequest) -> Result<Self, Self::Error> {
    let password: Option<SensitiveString> =
      Some(format!("{:?}{:?}", rand_number5(), rand_number5()).into());
    let username =
      get_required_sensitive(&form.email, FastJobErrorType::EmptyUsername)?.into_inner();
    let email = get_required_sensitive(&form.email, FastJobErrorType::EmptyEmail)?;
    let password = get_required_sensitive(&password, FastJobErrorType::EmptyPassword)?;
    // Check if email format is valid
    if !is_valid_email(&email) {
      return Err(FastJobErrorType::InvalidEmail.into());
    }
    Ok(Register {
      username,
      password,
      self_promotion: None,
      email: Some(email),
      captcha_uuid: None,
      captcha_answer: None,
      honeypot: None,
      answer: form.answer.take(),
      accepted_application: Some(false),
    })
  }
}
