use crate::BillingView;
use app_108jobs_db_schema::traits::Crud;
use app_108jobs_db_schema::{
  newtypes::{BillingId, LocalUserId},
  source::billing::Billing,
  utils::{get_conn, DbPool},
};
use app_108jobs_db_schema_file::schema::billing;
use app_108jobs_utils::error::FastJobResult;
/// Read-only view/query methods for Billing
impl BillingView {
  pub async fn read(pool: &mut DbPool<'_>, billing_id: BillingId) -> FastJobResult<Billing> {
    let billing = Billing::read(pool, billing_id).await?;
    Ok(billing)
  }
  /// ภายใน impl BillingView
  async fn list_by_user(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    by_employer: bool,
  ) -> FastJobResult<Vec<Billing>> {
    use diesel::prelude::*;
    let conn = &mut get_conn(pool).await?;
    // เลือก predicate ตามฝั่ง
    let mut q = billing::table.into_boxed();
    if by_employer {
      q = q.filter(billing::employer_id.eq(user_id));
    } else {
      q = q.filter(billing::freelancer_id.eq(user_id));
    }
    {
      let res = diesel_async::RunQueryDsl::load(q.order(billing::created_at.desc()), conn).await?;
      Ok(res)
    }
  }

  pub async fn list_by_freelancer(
    pool: &mut DbPool<'_>,
    freelancer_id: LocalUserId,
  ) -> FastJobResult<Vec<Billing>> {
    Self::list_by_user(pool, freelancer_id, false).await
  }

  pub async fn list_by_employer(
    pool: &mut DbPool<'_>,
    employer_id: LocalUserId,
  ) -> FastJobResult<Vec<Billing>> {
    Self::list_by_user(pool, employer_id, true).await
  }

}