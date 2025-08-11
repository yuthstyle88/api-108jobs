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
// ===== Typestate State Machine (structs-only) =====
// Each state is a distinct struct; allowed transitions are methods that
// consume the current state and return the next state's struct + a domain transition payload.

// Wallet side-effects described here; executed in apply_transition()
enum WalletAction {
  PayToEscrow { wallet_id: WalletId, amount: f64 },
  ReleaseToFreelancer { user_id: LocalUserId, amount: f64 },
}

// Domain transitions used by apply_transition()
struct FundEscrowTransition { pub form: BillingUpdateForm, pub wallet: WalletAction }
struct ReleaseToFreelancerTransition { pub form: BillingUpdateForm, pub wallet: WalletAction }
struct RequestRevisionTransition { pub form: BillingUpdateForm }
struct SubmitWorkTransition { pub form: BillingUpdateForm }
struct UpdateAfterRevisionTransition { pub form: BillingUpdateForm }
struct ReplaceDeliverableTransition { pub form: BillingUpdateForm }
struct EditTermsTransition { pub form: BillingUpdateForm }
struct CancelTransition { pub form: BillingUpdateForm }
struct PrevTransition { pub form: BillingUpdateForm }

// Planner enum: unifies all transition variants for the DB/apply layer
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

// Shared data snapshot for typestate transitions
#[derive(Clone, Debug)]
struct FlowData {
  billing_id: BillingId,
  employer_id: LocalUserId,
  freelancer_id: LocalUserId,
  amount: f64,
  revisions_used: i32,
  max_revisions: i32,
}

// ===== States as structs =====
#[derive(Debug)] struct QuotationPendingTS { data: FlowData }
#[derive(Debug)] struct OrderApprovedTS   { data: FlowData }
#[derive(Debug)] struct PaidEscrowTS      { data: FlowData }
#[derive(Debug)] struct WorkSubmittedTS   { data: FlowData }
#[derive(Debug)] struct RevisionRequestedTS { data: FlowData }
#[derive(Debug)] struct RequestChangeTS     { data: FlowData }
#[derive(Debug)] struct UpdatedTS         { data: FlowData }
#[derive(Debug)] struct CompletedTS       { data: FlowData }
#[derive(Debug)] struct CancelledTS       { data: FlowData }
#[derive(Debug)] struct DisputedTS        { data: FlowData }

// ===== Allowed transitions (methods consume self) =====
impl QuotationPendingTS {
  pub fn approve_and_fund(self, wallet_id: WalletId) -> Result<(PaidEscrowTS, FundEscrowTransition), FastJobErrorType> {
    let form = BillingUpdateForm {
      status: Some(BillingStatus::PaidEscrow),
      paid_at: Some(Some(Utc::now())),
      updated_at: Some(Utc::now()),
      ..Default::default()
    };
    let tx = FundEscrowTransition {
      form,
      wallet: WalletAction::PayToEscrow { wallet_id, amount: self.data.amount },
    };
    Ok((PaidEscrowTS { data: self.data }, tx))
  }
  pub fn cancel(self) -> (CancelledTS, CancelTransition) {
    let form = BillingUpdateForm { status: Some(BillingStatus::Cancelled), updated_at: Some(Utc::now()), ..Default::default() };
    (CancelledTS { data: self.data }, CancelTransition { form })
  }
}

impl OrderApprovedTS {
  pub fn pay_escrow(self, wallet_id: WalletId) -> Result<(PaidEscrowTS, FundEscrowTransition), FastJobErrorType> {
    let form = BillingUpdateForm { status: Some(BillingStatus::PaidEscrow), paid_at: Some(Some(Utc::now())), updated_at: Some(Utc::now()), ..Default::default() };
    let tx = FundEscrowTransition { form, wallet: WalletAction::PayToEscrow { wallet_id, amount: self.data.amount } };
    Ok((PaidEscrowTS { data: self.data }, tx))
  }
  pub fn cancel(self) -> (CancelledTS, CancelTransition) {
    let form = BillingUpdateForm { status: Some(BillingStatus::Cancelled), updated_at: Some(Utc::now()), ..Default::default() };
    (CancelledTS { data: self.data }, CancelTransition { form })
  }
}

impl PaidEscrowTS {
  pub fn submit_work(self, desc: String, url: Option<String>) -> Result<(WorkSubmittedTS, SubmitWorkTransition), FastJobErrorType> {
    if desc.trim().is_empty() { return Err(FastJobErrorType::InvalidField("work_description is required".into())); }
    let form = BillingUpdateForm { status: Some(BillingStatus::WorkSubmitted), work_description: Some(Some(desc)), deliverable_url: Some(url), updated_at: Some(Utc::now()), ..Default::default() };
    Ok((WorkSubmittedTS { data: self.data }, SubmitWorkTransition { form }))
  }
}

impl WorkSubmittedTS {
  pub fn request_revision(self, feedback: String) -> Result<(RevisionRequestedTS, RequestRevisionTransition), FastJobErrorType> {
    if feedback.trim().is_empty() { return Err(FastJobErrorType::InvalidField("revision_feedback is required".into())); }
    if self.data.revisions_used >= self.data.max_revisions { return Err(FastJobErrorType::InvalidField("Maximum revisions exceeded".into())); }
    let form = BillingUpdateForm { status: Some(BillingStatus::RequestChange), revision_feedback: Some(Some(feedback)), revisions_used: Some(self.data.revisions_used + 1), updated_at: Some(Utc::now()), ..Default::default() };
    Ok((RevisionRequestedTS { data: FlowData { revisions_used: self.data.revisions_used + 1, ..self.data } }, RequestRevisionTransition { form }))
  }
  pub fn approve_work(self) -> (CompletedTS, ReleaseToFreelancerTransition) {
    let form = BillingUpdateForm { status: Some(BillingStatus::Completed), updated_at: Some(Utc::now()), ..Default::default() };
    let tx = ReleaseToFreelancerTransition { form, wallet: WalletAction::ReleaseToFreelancer { user_id: self.data.freelancer_id, amount: self.data.amount } };
    (CompletedTS { data: self.data }, tx)
  }
}

impl RevisionRequestedTS {
  pub fn resubmit(self, desc: String, url: Option<String>) -> Result<(UpdatedTS, UpdateAfterRevisionTransition), FastJobErrorType> {
    if desc.trim().is_empty() { return Err(FastJobErrorType::InvalidField("updated_work_description is required".into())); }
    let form = BillingUpdateForm { status: Some(BillingStatus::Updated), work_description: Some(Some(desc)), deliverable_url: Some(url), updated_at: Some(Utc::now()), ..Default::default() };
    Ok((UpdatedTS { data: self.data }, UpdateAfterRevisionTransition { form }))
  }
}

impl RequestChangeTS {
  pub fn resubmit(self, desc: String, url: Option<String>) -> Result<(UpdatedTS, UpdateAfterRevisionTransition), FastJobErrorType> {
    if desc.trim().is_empty() { return Err(FastJobErrorType::InvalidField("updated_work_description is required".into())); }
    let form = BillingUpdateForm { status: Some(BillingStatus::Updated), work_description: Some(Some(desc)), deliverable_url: Some(url), updated_at: Some(Utc::now()), ..Default::default() };
    Ok((UpdatedTS { data: self.data }, UpdateAfterRevisionTransition { form }))
  }
}

impl UpdatedTS {
  pub fn request_revision(self, feedback: String) -> Result<(RevisionRequestedTS, RequestRevisionTransition), FastJobErrorType> {
    if feedback.trim().is_empty() { return Err(FastJobErrorType::InvalidField("revision_feedback is required".into())); }
    if self.data.revisions_used >= self.data.max_revisions { return Err(FastJobErrorType::InvalidField("Maximum revisions exceeded".into())); }
    let form = BillingUpdateForm { status: Some(BillingStatus::RequestChange), revision_feedback: Some(Some(feedback)), revisions_used: Some(self.data.revisions_used + 1), updated_at: Some(Utc::now()), ..Default::default() };
    Ok((RevisionRequestedTS { data: FlowData { revisions_used: self.data.revisions_used + 1, ..self.data } }, RequestRevisionTransition { form }))
  }
  pub fn approve_work(self) -> (CompletedTS, ReleaseToFreelancerTransition) {
    let form = BillingUpdateForm { status: Some(BillingStatus::Completed), updated_at: Some(Utc::now()), ..Default::default() };
    let tx = ReleaseToFreelancerTransition { form, wallet: WalletAction::ReleaseToFreelancer { user_id: self.data.freelancer_id, amount: self.data.amount } };
    (CompletedTS { data: self.data }, tx)
  }
}

// Terminal states (no outbound transitions)
impl CompletedTS {}
impl CancelledTS {}
impl DisputedTS {}

// Map Billing row → typestate
fn into_ts(b: &Billing) -> FlowData {
  FlowData {
    billing_id: b.id,
    employer_id: b.employer_id,
    freelancer_id: b.freelancer_id,
    amount: b.amount,
    revisions_used: b.revisions_used,
    max_revisions: b.max_revisions,
  }
}

// Helper: apply transitions (wallet + Crud update)
async fn apply_transition(pool: &mut DbPool<'_>, billing_id: BillingId, plan: Planned) -> FastJobResult<Billing> {
  // Wallet side-effects first (API expects &mut DbPool). If *_tx available, move inside txn.
  match &plan {
    Planned::FundEscrow(t) => { if let WalletAction::PayToEscrow { wallet_id, amount } = &t.wallet { WalletView::pay_for_job(pool, *wallet_id, *amount).await?; } }
    Planned::ReleaseToFreelancer(t) => { if let WalletAction::ReleaseToFreelancer { user_id, amount } = &t.wallet { WalletView::complete_job_payment(pool, *user_id, *amount).await?; } }
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
    let data = into_ts(&b);
    let plan = match b.status {
      BillingStatus::QuotationPending => {
        let ( _next, tx ) = QuotationPendingTS { data }.approve_and_fund(wallet_id)?;
        Planned::FundEscrow(tx)
      }
      BillingStatus::OrderApproved => {
        let ( _next, tx ) = OrderApprovedTS { data }.pay_escrow(wallet_id)?;
        Planned::FundEscrow(tx)
      }
      _ => return Err(FastJobErrorType::InvalidField("Billing not in a fundable state".into()).into()),
    };
    let updated = apply_transition(pool, billing_id, plan).await?;
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
    let data = into_ts(&b);
    let plan = match b.status {
      BillingStatus::PaidEscrow => {
        let ( _next, tx ) = PaidEscrowTS { data }.submit_work(work_description, deliverable_url)?;
        Planned::SubmitWork(tx)
      }
      _ => return Err(FastJobErrorType::InvalidField("Billing not in PaidEscrow".into()).into()),
    };
    let updated = apply_transition(pool, billing_id, plan).await?;
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
    let data = into_ts(&b);
    let plan = match b.status {
      BillingStatus::WorkSubmitted => {
        let ( _next, tx ) = WorkSubmittedTS { data }.request_revision(revision_feedback)?;
        Planned::RequestRevision(tx)
      }
      BillingStatus::Updated => {
        let ( _next, tx ) = UpdatedTS { data }.request_revision(revision_feedback)?;
        Planned::RequestRevision(tx)
      }
      _ => return Err(FastJobErrorType::InvalidField("Billing not ready for revision".into()).into()),
    };
    let updated = apply_transition(pool, billing_id, plan).await?;
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
    let data = into_ts(&b);
    let plan = match b.status {
      BillingStatus::RevisionRequested => {
        let ( _next, tx ) = RevisionRequestedTS { data }.resubmit(updated_work_description, updated_deliverable_url)?;
        Planned::UpdateAfterRevision(tx)
      }
      BillingStatus::RequestChange => {
        let ( _next, tx ) = RequestChangeTS { data }.resubmit(updated_work_description, updated_deliverable_url)?;
        Planned::UpdateAfterRevision(tx)
      }
      _ => return Err(FastJobErrorType::InvalidField("Billing not in revision state".into()).into()),
    };
    let updated = apply_transition(pool, billing_id, plan).await?;
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
    let data = into_ts(&b);
    let plan = match b.status {
      BillingStatus::WorkSubmitted => {
        let ( _next, tx ) = WorkSubmittedTS { data }.approve_work();
        Planned::ReleaseToFreelancer(tx)
      }
      BillingStatus::Updated => {
        let ( _next, tx ) = UpdatedTS { data }.approve_work();
        Planned::ReleaseToFreelancer(tx)
      }
      _ => return Err(FastJobErrorType::InvalidField("Billing not ready to approve".into()).into()),
    };
    let updated = apply_transition(pool, billing_id, plan).await?;
    Ok(updated)
  }
}