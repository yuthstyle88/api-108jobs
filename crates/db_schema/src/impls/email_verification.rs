use crate::{
  newtypes::LocalUserId,
  source::email_verification::{EmailVerification, EmailVerificationForm},
  utils::{get_conn, now, DbPool},
};
use diesel::{dsl::IntervalDsl, insert_into, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use app_108jobs_db_schema_file::schema::email_verification;
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl EmailVerification {
  pub async fn create(pool: &mut DbPool<'_>, form: &EmailVerificationForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(email_verification::table)
      .values(form)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateEmailVerification)
  }

  pub async fn read_for_code(pool: &mut DbPool<'_>, code: &str) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    email_verification::table
      .filter(email_verification::verification_code.eq(code))
      .filter(email_verification::published_at.gt(now() - 7.days()))
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
  pub async fn delete_old_codes_for_local_user(
    pool: &mut DbPool<'_>,
    local_user_id_: LocalUserId,
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      email_verification::table.filter(email_verification::local_user_id.eq(local_user_id_)),
    )
    .execute(conn)
    .await
    .with_fastjob_type(FastJobErrorType::Deleted)
  }
}
