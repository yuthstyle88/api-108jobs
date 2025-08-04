use crate::newtypes::LocalUserId;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::otp_verification;

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = otp_verification))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct OTPVerification {
  pub id: i32,
  pub local_user_id: LocalUserId,
  pub email: String,
  pub verification_otp: String,
  pub created_at: DateTime<Utc>,
}
#[derive(Clone, Debug)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = otp_verification))]
pub struct OTPVerificationForm {
  pub local_user_id: LocalUserId,
  pub email: String,
  pub verification_otp: Option<String>,
}
