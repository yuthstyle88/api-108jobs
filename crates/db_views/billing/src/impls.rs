use crate::BillingView;
use diesel::{prelude::*, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{LocalUserId, BillingId, PostId, CommentId},
  source::{
    billing::{Billing, BillingInsertForm},
  },
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::{enums::BillingStatus, schema::billing};
use lemmy_utils::error::{FastJobErrorType, FastJobResult};

impl BillingView {
  pub async fn read(pool: &mut DbPool<'_>, billing_id: BillingId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let billing = billing::table.find(billing_id).first::<Billing>(conn).await?;
    Ok(BillingView { billing })
  }

  pub async fn create_invoice(
    pool: &mut DbPool<'_>,
    freelancer_id: LocalUserId,
    employer_id: LocalUserId,
    post_id: PostId,
    comment_id: Option<CommentId>,
    price: f64,
    proposal: String,
    name: String,
    job_description: String,
    work_steps: Vec<String>,
    revise_times: i32,
    revise_description: String,
    working_days: i32,
    deliverables: Vec<String>,
    note: Option<String>,
    starting_day: String,
    delivery_day: String,
  ) -> FastJobResult<Billing> {
    let conn = &mut get_conn(pool).await?;
    
    if price <= 0.0 {
      return Err(FastJobErrorType::InvalidField("Price must be positive".to_string()))?;
    }

    if revise_times < 0 {
      return Err(FastJobErrorType::InvalidField("Revise times cannot be negative".to_string()))?;
    }

    if working_days <= 0 {
      return Err(FastJobErrorType::InvalidField("Working days must be positive".to_string()))?;
    }

    // Create detailed description from all quotation fields
    let detailed_description = format!(
      "=== QUOTATION: {} ===\n\nProposal:\n{}\n\nJob Description:\n{}\n\nWork Steps:\n{}\n\nDeliverables:\n{}\n\nRevision Policy:\n- Maximum revisions: {}\n- Revision description: {}\n\nTimeline:\n- Working days: {}\n- Starting day: {}\n- Delivery day: {}\n\n{}",
      name,
      proposal,
      job_description,
      work_steps.iter().enumerate().map(|(i, step)| format!("{}. {}", i + 1, step)).collect::<Vec<_>>().join("\n"),
      deliverables.iter().map(|item| format!("- {}", item)).collect::<Vec<_>>().join("\n"),
      revise_times,
      revise_description,
      working_days,
      starting_day,
      delivery_day,
      note.as_ref().map(|n| format!("Additional Notes:\n{}", n)).unwrap_or_default()
    );

    // Create the billing record
    let billing_form = BillingInsertForm {
      freelancer_id,
      employer_id,
      post_id,
      comment_id,
      amount: price,
      description: detailed_description,
      max_revisions: revise_times,
      revisions_used: Some(0),
      status: Some(BillingStatus::QuotationPending),
      created_at: None,
    };

    let billing = diesel::insert_into(billing::table)
      .values(&billing_form)
      .get_result::<Billing>(conn)
      .await?;

    Ok(billing)
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

  pub async fn approve_quotation(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    employer_id: LocalUserId,
  ) -> FastJobResult<Billing> {
    use lemmy_db_schema::source::billing::{BillingUpdateForm};
    use lemmy_db_views_wallet::WalletView;
    
    // First check if the billing exists and belongs to this employer
    let billing = {
      let conn = &mut get_conn(pool).await?;
      billing::table
        .find(billing_id)
        .filter(billing::employer_id.eq(employer_id))
        .filter(billing::status.eq(BillingStatus::QuotationPending))
        .first::<Billing>(conn)
        .await?
    };

    // Check employer has sufficient balance and deduct money from wallet
    let amount = billing.amount;
    WalletView::pay_for_job(pool, employer_id, amount).await?;

    // Update status to PaidEscrow (money is now in escrow)
    let update_form = BillingUpdateForm {
      status: Some(BillingStatus::PaidEscrow),
      paid_at: Some(Some(chrono::Utc::now())),
      updated_at: Some(chrono::Utc::now()),
      ..Default::default()
    };

    let updated_billing = {
      let conn = &mut get_conn(pool).await?;
      diesel::update(billing::table)
        .filter(billing::id.eq(billing_id))
        .set(&update_form)
        .get_result::<Billing>(conn)
        .await?
    };

    Ok(updated_billing)
  }

  pub async fn submit_work(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    freelancer_id: LocalUserId,
    work_description: String,
    deliverable_url: Option<String>,
  ) -> FastJobResult<Billing> {
    use lemmy_db_schema::source::billing::{BillingUpdateForm};
    let conn = &mut get_conn(pool).await?;
    
    // First check if the billing exists and belongs to this freelancer and is in PaidEscrow status
    let _billing = billing::table
      .find(billing_id)
      .filter(billing::freelancer_id.eq(freelancer_id))
      .filter(billing::status.eq(BillingStatus::PaidEscrow))
      .first::<Billing>(conn)
      .await?;

    // Update status to WorkSubmitted and add work details
    let update_form = BillingUpdateForm {
      status: Some(BillingStatus::WorkSubmitted),
      work_description: Some(Some(work_description)),
      deliverable_url: Some(deliverable_url),
      updated_at: Some(chrono::Utc::now()),
      ..Default::default()
    };

    let updated_billing = diesel::update(billing::table)
      .filter(billing::id.eq(billing_id))
      .set(&update_form)
      .get_result::<Billing>(conn)
      .await?;

    Ok(updated_billing)
  }

  pub async fn request_revision(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    employer_id: LocalUserId,
    revision_feedback: String,
  ) -> FastJobResult<Billing> {
    use lemmy_db_schema::source::billing::{BillingUpdateForm};
    let conn = &mut get_conn(pool).await?;
    
    // First check if the billing exists and belongs to this employer and work is submitted
    let billing = billing::table
      .find(billing_id)
      .filter(billing::employer_id.eq(employer_id))
      .filter(billing::status.eq(BillingStatus::WorkSubmitted))
      .first::<Billing>(conn)
      .await?;

    // Check if revisions are available
    if billing.revisions_used >= billing.max_revisions {
      return Err(FastJobErrorType::InvalidField("Maximum revisions exceeded".to_string()))?;
    }

    // Update status to RequestChange and add revision feedback
    let update_form = BillingUpdateForm {
      status: Some(BillingStatus::RequestChange),
      revision_feedback: Some(Some(revision_feedback)),
      revisions_used: Some(billing.revisions_used + 1),
      updated_at: Some(chrono::Utc::now()),
      ..Default::default()
    };

    let updated_billing = diesel::update(billing::table)
      .filter(billing::id.eq(billing_id))
      .set(&update_form)
      .get_result::<Billing>(conn)
      .await?;

    Ok(updated_billing)
  }

  pub async fn update_work_after_revision(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    freelancer_id: LocalUserId,
    updated_work_description: String,
    updated_deliverable_url: Option<String>,
  ) -> FastJobResult<Billing> {
    use lemmy_db_schema::source::billing::{BillingUpdateForm};
    let conn = &mut get_conn(pool).await?;
    
    // First check if the billing exists and belongs to this freelancer and is in RequestChange status
    let _billing = billing::table
      .find(billing_id)
      .filter(billing::freelancer_id.eq(freelancer_id))
      .filter(billing::status.eq(BillingStatus::RequestChange))
      .first::<Billing>(conn)
      .await?;

    // Update work description and set status to Updated
    let update_form = BillingUpdateForm {
      status: Some(BillingStatus::Updated),
      work_description: Some(Some(updated_work_description)),
      deliverable_url: updated_deliverable_url.map(Some),
      updated_at: Some(chrono::Utc::now()),
      ..Default::default()
    };

    let updated_billing = diesel::update(billing::table)
      .filter(billing::id.eq(billing_id))
      .set(&update_form)
      .get_result::<Billing>(conn)
      .await?;

    Ok(updated_billing)
  }

  pub async fn approve_work(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    employer_id: LocalUserId,
  ) -> FastJobResult<Billing> {
    use lemmy_db_schema::source::billing::{BillingUpdateForm};
    use lemmy_db_views_wallet::WalletView;
    
    // First check if the billing exists and belongs to this employer and work is submitted or updated
    let billing = {
      let conn = &mut get_conn(pool).await?;
      billing::table
        .find(billing_id)
        .filter(billing::employer_id.eq(employer_id))
        .filter(
          billing::status.eq(BillingStatus::WorkSubmitted)
            .or(billing::status.eq(BillingStatus::Updated))
        )
        .first::<Billing>(conn)
        .await?
    };

    let amount = billing.amount;
    let freelancer_id = billing.freelancer_id;

    // Release payment to freelancer
    WalletView::complete_job_payment(pool, freelancer_id, amount).await?;

    // Update status to Completed
    let update_form = BillingUpdateForm {
      status: Some(BillingStatus::Completed),
      updated_at: Some(chrono::Utc::now()),
      ..Default::default()
    };

    let updated_billing = {
      let conn = &mut get_conn(pool).await?;
      diesel::update(billing::table)
        .filter(billing::id.eq(billing_id))
        .set(&update_form)
        .get_result::<Billing>(conn)
        .await?
    };

    Ok(updated_billing)
  }
}