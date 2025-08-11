use chrono::Utc;
use diesel_async::scoped_futures::ScopedFutureExt;
use lemmy_db_schema::source::billing::BillingUpdateForm;
use lemmy_db_schema::traits::Crud;
use lemmy_db_schema::utils::get_conn;
use lemmy_db_schema::{
  newtypes::{BillingId, LocalUserId, WalletId},
  source::billing::{Billing, BillingFromQuotation, BillingInsertForm},
  utils::DbPool,
};
use lemmy_db_schema_file::enums::BillingStatus;
use lemmy_db_views_wallet::api::{CreateInvoiceForm, ValidCreateInvoice};
use lemmy_db_views_wallet::WalletView;
use lemmy_utils::error::FastJobErrorType;
use lemmy_utils::error::{FastJobErrorExt2, FastJobResult};

fn build_detailed_description(data: &CreateInvoiceForm) -> String {
  format!(
    "Invoice for project: {}\nDetails: {}\nAmount: {}\nDue date: {}",
    data.project_name,
    data.project_details,
    data.amount,
    data.delivery_day
  )
}

/// Workflow/command operations for billing lifecycle (create, approve, submit, revise, complete).
pub struct WorkFlowService;

impl WorkFlowService {
  pub async fn validate_before_approve(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    employer_id: LocalUserId,
  ) -> FastJobResult<Billing> {
    let b = Billing::read(pool, billing_id).await?;
    if b.employer_id != employer_id {
      return Err(FastJobErrorType::InvalidField("Not the employer of this billing".into()).into());
    }
    if b.status == BillingStatus::Completed { return Ok(b); }
    if b.status != BillingStatus::WorkSubmitted && b.status != BillingStatus::Updated {
      return Err(FastJobErrorType::InvalidField("Billing not ready to approve".into()).into());
    }
    Ok(b)
  }

  pub async fn create_invoice(
    pool: &mut DbPool<'_>,
    freelancer_id: LocalUserId,
    data: ValidCreateInvoice,
  ) -> FastJobResult<Billing> {
    let inner = data.0;
    let description = build_detailed_description(&inner);
    let billing_form = BillingFromQuotation {
      employer_id: inner.employer_id,
      freelancer_id,
      description,
      amount: inner.amount,
      delivery_day: inner.delivery_day,
      ..Default::default()
    };
    let billing_form: BillingInsertForm = billing_form.into();
    Billing::create(pool, &billing_form).await
  }

  pub async fn approve_quotation(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    employer_id: LocalUserId,
    wallet_id: WalletId,
  ) -> FastJobResult<Billing> {
    let conn = &mut get_conn(pool).await?;
    // 1) Read the billing, ensure it belongs to employer and is QuotationPending.
    let billing = Self::validate_before_approve(&mut conn.into(), billing_id, employer_id).await?;

    // 2) Move funds to escrow.
    WalletView::pay_for_job(&mut conn.into(), wallet_id, billing.amount).await?;

    // 3) Build update form.
    let update_form = BillingUpdateForm {
      status: Some(BillingStatus::PaidEscrow),
      paid_at: Some(Some(Utc::now())),
      updated_at: Some(Utc::now()),
      ..Default::default()
    };
    // 4) Update billing in a transaction, using Crud path.
    let updated = conn
    .run_transaction(|conn| {
      async move {
        Billing::update(&mut conn.into(), billing_id, &update_form)
        .await
        .with_fastjob_type(FastJobErrorType::CouldntUpdateBilling)
      }
      .scope_boxed()
    })
    .await?;

    Ok(updated)
  }

  pub async fn submit_work(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    freelancer_id: LocalUserId,
    work_description: String,
    deliverable_url: Option<String>,
  ) -> FastJobResult<Billing> {
    let conn = &mut get_conn(pool).await?;
    // 1) Read billing and validate ownership & status
    let billing = Billing::read(&mut conn.into(), billing_id).await?;
    if billing.freelancer_id != freelancer_id {
      return Err(FastJobErrorType::InvalidField("Not the freelancer of this billing".into()).into());
    }
    if billing.status != BillingStatus::PaidEscrow {
      return Err(FastJobErrorType::InvalidField("Billing not in PaidEscrow status".into()).into());
    }
    // 2) Update to WorkSubmitted within a transaction
    let updated_billing = conn
    .run_transaction(|conn| {
      async move {
        let form = BillingUpdateForm {
          status: Some(BillingStatus::WorkSubmitted),
          work_description: Some(Some(work_description)),
          deliverable_url: Some(deliverable_url),
          updated_at: Some(chrono::Utc::now()),
          ..Default::default()
        };
        let b = Billing::update(&mut conn.into(), billing_id, &form).await
        .with_fastjob_type(FastJobErrorType::CouldntUpdateBilling)?;
        Ok(b)
      }
      .scope_boxed()
    })
    .await?;

    Ok(updated_billing)
  }

  pub async fn request_revision(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    employer_id: LocalUserId,
    revision_feedback: String,
  ) -> FastJobResult<Billing> {
    let conn = &mut get_conn(pool).await?;
    // อ่านบิล + ตรวจสิทธิ์/สถานะ
    let billing = Billing::read(&mut conn.into(), billing_id).await?;
    if billing.employer_id != employer_id {
      return Err(FastJobErrorType::InvalidField("Not the employer of this billing".into()).into());
    }
    if billing.status != BillingStatus::WorkSubmitted && billing.status != BillingStatus::Updated {
      return Err(FastJobErrorType::InvalidField("Billing not ready for revision".into()).into());
    }
    if billing.revisions_used >= billing.max_revisions {
      return Err(FastJobErrorType::InvalidField("Maximum revisions exceeded".to_string()).into());
    }

    // อัปเดตเป็น RequestChange ในทรานแซกชัน
    let updated_billing = {

      conn.run_transaction(|conn| {
        async move {
          let form = BillingUpdateForm {
            status: Some(BillingStatus::RequestChange),
            revision_feedback: Some(Some(revision_feedback)),
            revisions_used: Some(billing.revisions_used + 1),
            updated_at: Some(chrono::Utc::now()),
            ..Default::default()
          };
          let b = Billing::update(&mut conn.into(), billing_id, &form)
          .await
          .with_fastjob_type(FastJobErrorType::CouldntUpdateBilling)?;
          Ok(b)
        }
        .scope_boxed()
      }).await?
    };

    Ok(updated_billing)
  }

  pub async fn update_work_after_revision(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    freelancer_id: LocalUserId,
    updated_work_description: String,
    updated_deliverable_url: Option<String>,
  ) -> FastJobResult<Billing> {
    let conn = &mut get_conn(pool).await?;
    // 1) อ่านบิลและตรวจสิทธิ์/สถานะ
    let billing = Billing::read(&mut conn.into(), billing_id).await?;
    if billing.freelancer_id != freelancer_id {
      return Err(FastJobErrorType::InvalidField("Not the freelancer of this billing".into()).into());
    }
    if billing.status != BillingStatus::RequestChange {
      return Err(FastJobErrorType::InvalidField("Billing not in RequestChange status".into()).into());
    }

    // 2) อัปเดตงานหลังแก้ไขในทรานแซกชัน
    let updated_billing = conn
    .run_transaction(|conn| {
      async move {
        let form = BillingUpdateForm {
          status: Some(BillingStatus::Updated),
          work_description: Some(Some(updated_work_description)),
          deliverable_url: updated_deliverable_url.map(Some),
          updated_at: Some(chrono::Utc::now()),
          ..Default::default()
        };
        let b = Billing::update(&mut conn.into(), billing_id, &form)
        .await
        .with_fastjob_type(FastJobErrorType::CouldntUpdateBilling)?;
        Ok(b)
      }
      .scope_boxed()
    })
    .await?;

    Ok(updated_billing)
  }

  pub async fn approve_work(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    employer_id: LocalUserId,
  ) -> FastJobResult<Billing> {
    let conn = &mut get_conn(pool).await?;
    // อ่านบิล
    let billing = Self::validate_before_approve(&mut conn.into(), billing_id, employer_id).await?;
    let amount = billing.amount;
    let freelancer_id = billing.freelancer_id;

    // จ่ายเงินให้ฟรีแลนซ์
    WalletView::complete_job_payment(&mut conn.into(), freelancer_id, amount).await?;

    let updated_billing = {

      conn.run_transaction(|conn| {
        async move {
          let form = BillingUpdateForm {
            status: Some(BillingStatus::Completed),
            updated_at: Some(chrono::Utc::now()),
            ..Default::default()
          };
          let b = Billing::update(&mut conn.into(), billing_id, &form)
          .await
          .with_fastjob_type(FastJobErrorType::CouldntUpdateBilling)?;
          Ok(b)
        }
        .scope_boxed()
      }).await?
    };

    Ok(updated_billing)
  }
}