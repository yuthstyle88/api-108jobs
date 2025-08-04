use crate::{
  newtypes::LocalUserId,
  source::otp_verification::{OTPVerification, OTPVerificationForm},
  utils::{get_conn, now, DbPool},
};
use diesel::{dsl::IntervalDsl, insert_into, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::otp_verification;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl OTPVerification {
  pub async fn create(pool: &mut DbPool<'_>, form: &OTPVerificationForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(otp_verification::table)
      .values(form)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateOTPVerification)
  }

  pub async fn read_for_otp(pool: &mut DbPool<'_>, otp: &str) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    otp_verification::table
      .filter(otp_verification::verification_otp.eq(otp))
      .filter(otp_verification::created_at.gt(now() - 1.days()))
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
  pub async fn delete_old_otps_for_local_user(
    pool: &mut DbPool<'_>,
    local_user_id_: LocalUserId,
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      otp_verification::table.filter(otp_verification::local_user_id.eq(local_user_id_)),
    )
    .execute(conn)
    .await
    .with_fastjob_type(FastJobErrorType::Deleted)
  }
}
