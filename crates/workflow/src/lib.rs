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
// ===== State Machine Core =====
// Side-effects for wallet operations (described but not executed here)
enum WalletAction {
  PayToEscrow { wallet_id: WalletId, amount: f64 },
  ReleaseToFreelancer { user_id: LocalUserId, amount: f64 },
}

// Domain-specific transitions
struct FundEscrowTransition { pub form: BillingUpdateForm, pub wallet: WalletAction }
struct ReleaseToFreelancerTransition { pub form: BillingUpdateForm, pub wallet: WalletAction }
struct RequestRevisionTransition { pub form: BillingUpdateForm }
struct SubmitWorkTransition { pub form: BillingUpdateForm }
struct UpdateAfterRevisionTransition { pub form: BillingUpdateForm }
struct ReplaceDeliverableTransition { pub form: BillingUpdateForm }
struct EditTermsTransition { pub form: BillingUpdateForm }
struct CancelTransition { pub form: BillingUpdateForm }
struct PrevTransition { pub form: BillingUpdateForm }

enum Planned {
  FundEscrow(FundEscrowTransition),
  ReleaseToFreelancer(ReleaseToFreelancerTransition),
  RequestRevision(RequestRevisionTransition),
  SubmitWork(SubmitWorkTransition),
  UpdateAfterRevision(UpdateAfterRevisionTransition),
  ReplaceDeliverable(ReplaceDeliverableTransition),
  EditTerms(EditTermsTransition),
  Cancel(CancelTransition),
  Prev(PrevTransition),
}

// Snapshot (pure) of billing data that states need
#[derive(Clone)]
struct Snapshot {
  billing_id: BillingId,
  employer_id: LocalUserId,
  freelancer_id: LocalUserId,
  status: BillingStatus,
  revisions_used: i32,
  max_revisions: i32,
  amount: f64,
}

trait BillingState {
  fn snap(&self) -> &Snapshot;

  fn approve_quotation(&self, _wallet_id: WalletId) -> Result<FundEscrowTransition, FastJobErrorType> {
    Err(FastJobErrorType::InvalidField("Not allowed".into()))
  }
  fn submit_work(&self, _desc: String, _url: Option<String>) -> Result<SubmitWorkTransition, FastJobErrorType> {
    Err(FastJobErrorType::InvalidField("Not allowed".into()))
  }
  fn request_revision(&self, _feedback: String) -> Result<RequestRevisionTransition, FastJobErrorType> {
    Err(FastJobErrorType::InvalidField("Not allowed".into()))
  }
  fn update_after_revision(&self, _desc: String, _url: Option<String>) -> Result<UpdateAfterRevisionTransition, FastJobErrorType> {
    Err(FastJobErrorType::InvalidField("Not allowed".into()))
  }
  fn approve_work(&self) -> Result<ReleaseToFreelancerTransition, FastJobErrorType> {
    Err(FastJobErrorType::InvalidField("Not allowed".into()))
  }
  fn cancel(&self) -> Result<CancelTransition, FastJobErrorType> {
    Err(FastJobErrorType::InvalidField("Not allowed".into()))
  }
  fn prev(&self) -> Result<PrevTransition, FastJobErrorType> {
    Err(FastJobErrorType::InvalidField("Not allowed".into()))
  }
}

// ===== Concrete States =====
struct QuotationPending { s: Snapshot }
impl BillingState for QuotationPending {
  fn snap(&self) -> &Snapshot { &self.s }
  fn approve_quotation(&self, wallet_id: WalletId) -> Result<FundEscrowTransition, FastJobErrorType> {
    Ok(FundEscrowTransition{ form: BillingUpdateForm{
      status: Some(BillingStatus::PaidEscrow),
      paid_at: Some(Some(Utc::now())),
      updated_at: Some(Utc::now()),
      ..Default::default()
    }, wallet: WalletAction::PayToEscrow{ wallet_id, amount: self.s.amount } })
  }
  fn cancel(&self) -> Result<CancelTransition, FastJobErrorType> {
    Ok(CancelTransition{ form: BillingUpdateForm{
      status: Some(BillingStatus::Cancelled),
      updated_at: Some(Utc::now()),
      ..Default::default()
    } })
  }
}

struct OrderApproved { s: Snapshot }
impl BillingState for OrderApproved {
  fn snap(&self) -> &Snapshot { &self.s }
  fn approve_quotation(&self, wallet_id: WalletId) -> Result<FundEscrowTransition, FastJobErrorType> {
    Ok(FundEscrowTransition{ form: BillingUpdateForm{
      status: Some(BillingStatus::PaidEscrow),
      paid_at: Some(Some(Utc::now())),
      updated_at: Some(Utc::now()),
      ..Default::default()
    }, wallet: WalletAction::PayToEscrow{ wallet_id, amount: self.s.amount } })
  }
  fn cancel(&self) -> Result<CancelTransition, FastJobErrorType> {
    Ok(CancelTransition{ form: BillingUpdateForm{
      status: Some(BillingStatus::Cancelled),
      updated_at: Some(Utc::now()),
      ..Default::default()
    } })
  }
}

struct PaidEscrow { s: Snapshot }
impl BillingState for PaidEscrow {
  fn snap(&self) -> &Snapshot { &self.s }
  fn submit_work(&self, desc: String, url: Option<String>) -> Result<SubmitWorkTransition, FastJobErrorType> {
    Ok(SubmitWorkTransition{ form: BillingUpdateForm{
      status: Some(BillingStatus::WorkSubmitted),
      work_description: Some(Some(desc)),
      deliverable_url: Some(url),
      updated_at: Some(Utc::now()),
      ..Default::default()
    } })
  }
}

struct WorkSubmitted { s: Snapshot }
impl BillingState for WorkSubmitted {
  fn snap(&self) -> &Snapshot { &self.s }
  fn request_revision(&self, feedback: String) -> Result<RequestRevisionTransition, FastJobErrorType> {
    if self.s.revisions_used >= self.s.max_revisions {
      return Err(FastJobErrorType::InvalidField("Maximum revisions exceeded".into()));
    }
    Ok(RequestRevisionTransition{ form: BillingUpdateForm{
      status: Some(BillingStatus::RequestChange),
      revision_feedback: Some(Some(feedback)),
      revisions_used: Some(self.s.revisions_used + 1),
      updated_at: Some(Utc::now()),
      ..Default::default()
    } })
  }
  fn approve_work(&self) -> Result<ReleaseToFreelancerTransition, FastJobErrorType> {
    Ok(ReleaseToFreelancerTransition{ form: BillingUpdateForm{
      status: Some(BillingStatus::Completed),
      updated_at: Some(Utc::now()),
      ..Default::default()
    }, wallet: WalletAction::ReleaseToFreelancer{ user_id: self.s.freelancer_id, amount: self.s.amount } })
  }
  fn prev(&self) -> Result<PrevTransition, FastJobErrorType> {
    Ok(PrevTransition{ form: BillingUpdateForm{
      status: Some(BillingStatus::PaidEscrow),
      updated_at: Some(Utc::now()),
      ..Default::default()
    } })
  }
}

struct RequestChange { s: Snapshot }
impl BillingState for RequestChange {
  fn snap(&self) -> &Snapshot { &self.s }
  fn update_after_revision(&self, desc: String, url: Option<String>) -> Result<UpdateAfterRevisionTransition, FastJobErrorType> {
    Ok(UpdateAfterRevisionTransition{ form: BillingUpdateForm{
      status: Some(BillingStatus::Updated),
      work_description: Some(Some(desc)),
      deliverable_url: Some(url),
      updated_at: Some(Utc::now()),
      ..Default::default()
    } })
  }
  fn prev(&self) -> Result<PrevTransition, FastJobErrorType> {
    Ok(PrevTransition{ form: BillingUpdateForm{
      status: Some(BillingStatus::WorkSubmitted),
      updated_at: Some(Utc::now()),
      ..Default::default()
    } })
  }
}

struct RevisionRequested { s: Snapshot }
impl BillingState for RevisionRequested {
  fn snap(&self) -> &Snapshot { &self.s }
  fn update_after_revision(&self, desc: String, url: Option<String>) -> Result<UpdateAfterRevisionTransition, FastJobErrorType> {
    Ok(UpdateAfterRevisionTransition{ form: BillingUpdateForm{
      status: Some(BillingStatus::Updated),
      work_description: Some(Some(desc)),
      deliverable_url: Some(url),
      updated_at: Some(Utc::now()),
      ..Default::default()
    } })
  }
  fn prev(&self) -> Result<PrevTransition, FastJobErrorType> {
    Ok(PrevTransition{ form: BillingUpdateForm{
      status: Some(BillingStatus::WorkSubmitted),
      updated_at: Some(Utc::now()),
      ..Default::default()
    } })
  }
}

struct Updated { s: Snapshot }
impl BillingState for Updated {
  fn snap(&self) -> &Snapshot { &self.s }
  fn request_revision(&self, feedback: String) -> Result<RequestRevisionTransition, FastJobErrorType> {
    if self.s.revisions_used >= self.s.max_revisions {
      return Err(FastJobErrorType::InvalidField("Maximum revisions exceeded".into()));
    }
    Ok(RequestRevisionTransition{ form: BillingUpdateForm{
      status: Some(BillingStatus::RequestChange),
      revision_feedback: Some(Some(feedback)),
      revisions_used: Some(self.s.revisions_used + 1),
      updated_at: Some(Utc::now()),
      ..Default::default()
    } })
  }
  fn approve_work(&self) -> Result<ReleaseToFreelancerTransition, FastJobErrorType> {
    Ok(ReleaseToFreelancerTransition{ form: BillingUpdateForm{
      status: Some(BillingStatus::Completed),
      updated_at: Some(Utc::now()),
      ..Default::default()
    }, wallet: WalletAction::ReleaseToFreelancer{ user_id: self.s.freelancer_id, amount: self.s.amount } })
  }
  fn prev(&self) -> Result<PrevTransition, FastJobErrorType> {
    Ok(PrevTransition{ form: BillingUpdateForm{
      status: Some(BillingStatus::RequestChange),
      updated_at: Some(Utc::now()),
      ..Default::default()
    } })
  }
}

// Terminal state for completed or cancelled billings (no ops allowed)
struct Terminal { s: Snapshot }
impl BillingState for Terminal {
  fn snap(&self) -> &Snapshot { &self.s }
}

// Map a Billing row to its state struct
fn to_state(b: &Billing) -> Box<dyn BillingState> {
  let s = Snapshot {
    billing_id: b.id,
    employer_id: b.employer_id,
    freelancer_id: b.freelancer_id,
    status: b.status,
    revisions_used: b.revisions_used,
    max_revisions: b.max_revisions,
    amount: b.amount,
  };
  match b.status {
    BillingStatus::QuotationPending   => Box::new(QuotationPending { s }),
    BillingStatus::OrderApproved      => Box::new(OrderApproved { s }),
    BillingStatus::PaidEscrow         => Box::new(PaidEscrow { s }),
    BillingStatus::WorkSubmitted      => Box::new(WorkSubmitted { s }),
    BillingStatus::RevisionRequested  => Box::new(RevisionRequested { s }),
    BillingStatus::RequestChange      => Box::new(RequestChange { s }),
    BillingStatus::Updated            => Box::new(Updated { s }),
    BillingStatus::Completed          => Box::new(Terminal { s }),
    BillingStatus::Disputed           => Box::new(Terminal { s }),
    BillingStatus::Cancelled          => Box::new(Terminal { s }),
  }
}

// Helper to apply transitions (wallet action + DB update via Crud) consistently
async fn apply_transition(pool: &mut DbPool<'_>, billing_id: BillingId, plan: Planned) -> FastJobResult<Billing> {
  match &plan {
    Planned::FundEscrow(t) => {
      if let WalletAction::PayToEscrow { wallet_id, amount } = &t.wallet {
        WalletView::pay_for_job(pool, *wallet_id, *amount).await?;
      }
    }
    Planned::ReleaseToFreelancer(t) => {
      if let WalletAction::ReleaseToFreelancer { user_id, amount } = &t.wallet {
        WalletView::complete_job_payment(pool, *user_id, *amount).await?;
      }
    }
    _ => {}
  }

  let form = match &plan {
    Planned::FundEscrow(t) => &t.form,
    Planned::ReleaseToFreelancer(t) => &t.form,
    Planned::RequestRevision(t) => &t.form,
    Planned::SubmitWork(t) => &t.form,
    Planned::UpdateAfterRevision(t) => &t.form,
    Planned::ReplaceDeliverable(t) => &t.form,
    Planned::EditTerms(t) => &t.form,
    Planned::Cancel(t) => &t.form,
    Planned::Prev(t) => &t.form,
  };

  let conn = &mut get_conn(pool).await?;
  let updated = conn
    .run_transaction(|conn| {
      async move {
        let b = Billing::update(&mut conn.into(), billing_id, form)
          .await
          .with_fastjob_type(FastJobErrorType::CouldntUpdateBilling)?;
        Ok(b)
      }
      .scope_boxed()
    })
    .await?;

  Ok(updated)
}

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
    let b = Billing::read(pool, billing_id).await?;
    if b.employer_id != employer_id {
      return Err(FastJobErrorType::InvalidField("Not the employer of this billing".into()).into());
    }
    let plan = to_state(&b).approve_quotation(wallet_id)?; // FundEscrowTransition
    let updated = apply_transition(pool, billing_id, Planned::FundEscrow(plan)).await?;
    Ok(updated)
  }

  pub async fn submit_work(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    freelancer_id: LocalUserId,
    work_description: String,
    deliverable_url: Option<String>,
  ) -> FastJobResult<Billing> {
    let b = Billing::read(pool, billing_id).await?;
    if b.freelancer_id != freelancer_id {
      return Err(FastJobErrorType::InvalidField("Not the freelancer of this billing".into()).into());
    }
    let plan = to_state(&b).submit_work(work_description, deliverable_url)?;
    let updated = apply_transition(pool, billing_id, Planned::SubmitWork(plan)).await?;
    Ok(updated)
  }

  pub async fn request_revision(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    employer_id: LocalUserId,
    revision_feedback: String,
  ) -> FastJobResult<Billing> {
    let b = Billing::read(pool, billing_id).await?;
    if b.employer_id != employer_id {
      return Err(FastJobErrorType::InvalidField("Not the employer of this billing".into()).into());
    }
    let plan = to_state(&b).request_revision(revision_feedback)?;
    let updated = apply_transition(pool, billing_id, Planned::RequestRevision(plan)).await?;
    Ok(updated)
  }

  pub async fn update_work_after_revision(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    freelancer_id: LocalUserId,
    updated_work_description: String,
    updated_deliverable_url: Option<String>,
  ) -> FastJobResult<Billing> {
    let b = Billing::read(pool, billing_id).await?;
    if b.freelancer_id != freelancer_id {
      return Err(FastJobErrorType::InvalidField("Not the freelancer of this billing".into()).into());
    }
    let plan = to_state(&b).update_after_revision(updated_work_description, updated_deliverable_url)?;
    let updated = apply_transition(pool, billing_id, Planned::UpdateAfterRevision(plan)).await?;
    Ok(updated)
  }

  pub async fn approve_work(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    employer_id: LocalUserId,
  ) -> FastJobResult<Billing> {
    let b = Billing::read(pool, billing_id).await?;
    if b.employer_id != employer_id {
      return Err(FastJobErrorType::InvalidField("Not the employer of this billing".into()).into());
    }
    let plan = to_state(&b).approve_work()?; // ReleaseToFreelancerTransition
    let updated = apply_transition(pool, billing_id, Planned::ReleaseToFreelancer(plan)).await?;
    Ok(updated)
  }
}