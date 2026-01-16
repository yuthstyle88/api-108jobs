use crate::sensitive::SensitiveString;
#[cfg(feature = "full")]
use app_108jobs_db_schema_file::schema::secret;

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = secret))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct Secret {
  pub id: i32,
  pub jwt_secret: SensitiveString,
}
