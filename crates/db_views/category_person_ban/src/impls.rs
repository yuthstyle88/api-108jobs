use crate::CategoryPersonBanView;
use app_108jobs_core::error::{FastJobErrorType, FastJobResult};
use app_108jobs_db::{
  newtypes::{CategoryId, PersonId},
  schema::category_actions,
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{exists, not},
  select,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

impl CategoryPersonBanView {
  pub async fn check(
    pool: &mut DbPool<'_>,
    from_person_id: PersonId,
    from_category_id: CategoryId,
  ) -> FastJobResult<()> {
    let conn = &mut get_conn(pool).await?;
    let find_action = category_actions::table
      .find((from_person_id, from_category_id))
      .filter(category_actions::received_ban_at.is_not_null());
    select(not(exists(find_action)))
      .get_result::<bool>(conn)
      .await?
      .then_some(())
      .ok_or(FastJobErrorType::PersonIsBannedFromCategory.into())
  }
}
