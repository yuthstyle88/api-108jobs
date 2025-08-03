use crate::newtypes::{ActivityId, DbUrl};
use crate::{
  source::activity::{SentActivity, SentActivityForm},
  utils::{get_conn, DbPool},
};
use diesel::dsl::insert_into;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use std::fmt::Debug;

impl SentActivity {
  pub async fn create(pool: &mut DbPool<'_>, form: SentActivityForm) -> FastJobResult<Self> {
    use lemmy_db_schema_file::schema::sent_activity::dsl::sent_activity;
    let conn = &mut get_conn(pool).await?;
    insert_into(sent_activity)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntInsertActivity)
  }

  pub async fn read_from_apub_id(pool: &mut DbPool<'_>, object_id: &DbUrl) -> FastJobResult<Self> {
    use lemmy_db_schema_file::schema::sent_activity::dsl::{ap_id, sent_activity};
    let conn = &mut get_conn(pool).await?;
    sent_activity
     .filter(ap_id.eq(object_id))
     .first(conn)
     .await
     .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn read(pool: &mut DbPool<'_>, object_id: ActivityId) -> FastJobResult<Self> {
    use lemmy_db_schema_file::schema::sent_activity::dsl::sent_activity;
    let conn = &mut get_conn(pool).await?;
    sent_activity
      .find(object_id)
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}


