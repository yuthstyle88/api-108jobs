use crate::BillingView;
use diesel::{prelude::*, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{LocalUserId, BillingId},
  source::{
    billing::Billing,
  },
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::{schema::billing};
use lemmy_utils::error::FastJobResult;
use lemmy_db_schema::newtypes::WalletId;
use lemmy_db_views_wallet::api::ValidCreateInvoice;

/// Read-only view/query methods for Billing
impl BillingView {
  pub async fn read(pool: &mut DbPool<'_>, billing_id: BillingId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let billing = billing::table.find(billing_id).first::<Billing>(conn).await?;
    Ok(BillingView { billing })
  }

  pub async fn list_by_freelancer(
    pool: &mut DbPool<'_>,
    freelancer_id: LocalUserId,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    let billings = billing::table
      .filter(billing::freelancer_id.eq(freelancer_id))
      .order(billing::created_at.desc())
      .load::<Billing>(conn)
      .await?;

    Ok(billings.into_iter().map(|billing| BillingView { billing }).collect())
  }

  pub async fn list_by_employer(
    pool: &mut DbPool<'_>,
    employer_id: LocalUserId,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    let billings = billing::table
      .filter(billing::employer_id.eq(employer_id))
      .order(billing::created_at.desc())
      .load::<Billing>(conn)
      .await?;

    Ok(billings.into_iter().map(|billing| BillingView { billing }).collect())
  }

  // Delegations to workflow service for command operations
  pub async fn create_invoice(
    pool: &mut DbPool<'_>,
    freelancer_id: LocalUserId,
    data: ValidCreateInvoice,
  ) -> FastJobResult<lemmy_db_schema::source::billing::Billing> {
    lemmy_workflow::WorkFlowService::create_invoice(pool, freelancer_id, data).await
  }

  pub async fn approve_quotation(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    employer_id: LocalUserId,
    wallet_id: WalletId,
  ) -> FastJobResult<lemmy_db_schema::source::billing::Billing> {
    lemmy_workflow::WorkFlowService::approve_quotation(pool, billing_id, employer_id, wallet_id).await
  }

  pub async fn submit_work(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    freelancer_id: LocalUserId,
    work_description: String,
    deliverable_url: Option<String>,
  ) -> FastJobResult<lemmy_db_schema::source::billing::Billing> {
    lemmy_workflow::WorkFlowService::submit_work(pool, billing_id, freelancer_id, work_description, deliverable_url).await
  }

  pub async fn request_revision(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    employer_id: LocalUserId,
    revision_feedback: String,
  ) -> FastJobResult<lemmy_db_schema::source::billing::Billing> {
    lemmy_workflow::WorkFlowService::request_revision(pool, billing_id, employer_id, revision_feedback).await
  }

  pub async fn update_work_after_revision(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    freelancer_id: LocalUserId,
    updated_work_description: String,
    updated_deliverable_url: Option<String>,
  ) -> FastJobResult<lemmy_db_schema::source::billing::Billing> {
    lemmy_workflow::WorkFlowService::update_work_after_revision(pool, billing_id, freelancer_id, updated_work_description, updated_deliverable_url).await
  }

  pub async fn approve_work(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    employer_id: LocalUserId,
  ) -> FastJobResult<lemmy_db_schema::source::billing::Billing> {
    lemmy_workflow::WorkFlowService::approve_work(pool, billing_id, employer_id).await
  }
}