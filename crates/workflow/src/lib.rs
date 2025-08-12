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
use lemmy_db_schema::source::wallet::Wallet;
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

// ===== Small helpers to reduce duplication =====
fn form_paid_escrow() -> BillingUpdateForm {
  BillingUpdateForm {
    status: Some(BillingStatus::PaidEscrow),
    paid_at: Some(Some(Utc::now())),
    updated_at: Some(Utc::now()),
    ..Default::default()
  }
}

#[allow(dead_code)]
fn form_cancelled() -> BillingUpdateForm {
  BillingUpdateForm { status: Some(BillingStatus::Cancelled), updated_at: Some(Utc::now()), ..Default::default() }
}


fn form_submit_work(desc: String, url: Option<String>) -> Result<BillingUpdateForm, FastJobErrorType> {
  if desc.trim().is_empty() {
    return Err(FastJobErrorType::InvalidField("work_description is required".into()));
  }
  Ok(BillingUpdateForm {
    status: Some(BillingStatus::WorkSubmitted),
    work_description: Some(Some(desc)),
    deliverable_url: Some(url),
    updated_at: Some(Utc::now()),
    ..Default::default()
  })
}

fn form_touch_only() -> BillingUpdateForm {
  BillingUpdateForm {
    updated_at: Some(Utc::now()),
    ..Default::default()
  }
}

fn form_completed_and_wallet(
  freelancer_id: LocalUserId,
  amount: f64,
) -> (BillingUpdateForm, WalletAction) {
  let form = BillingUpdateForm { status: Some(BillingStatus::Completed), updated_at: Some(Utc::now()), ..Default::default() };
  let wallet = WalletAction::ReleaseToFreelancer { user_id: freelancer_id, amount };
  (form, wallet)
}

/// Workflow/command operations for billing lifecycle (create, approve, submit, revise, complete).
// ===== Typestate State Machine (structs-only) =====
// Each state is a distinct struct; allowed transitions are methods that
// consume the current state and return the next state's struct + a domain transition payload.

// Wallet side-effects described here; executed in apply_transition()
#[allow(dead_code)]
enum WalletAction {
  PayToEscrow { wallet_id: WalletId, amount: f64 },
  ReleaseToFreelancer { user_id: LocalUserId, amount: f64 },
  RefundToEmployer { user_id: LocalUserId, amount: f64 }, // reserved for future use
}

// Domain transitions used by apply_transition()
struct FundEscrowTransition { pub form: BillingUpdateForm, pub wallet: WalletAction }
struct ReleaseToFreelancerTransition { pub form: BillingUpdateForm, pub wallet: WalletAction }
struct ApproveMilestoneTransition { pub form: BillingUpdateForm, pub wallet: WalletAction }
struct ReleaseRemainingTransition { pub form: BillingUpdateForm, pub wallet: WalletAction }
struct SubmitWorkTransition { pub form: BillingUpdateForm }
struct CancelTransition { #[allow(dead_code)] pub form: BillingUpdateForm }
// NOTE: No rollback (prev) transitions are supported. To restart, cancel this billing and open a new one.

// Planner enum: unifies all transition variants for the DB/apply layer
enum Planned {
  FundEscrow(FundEscrowTransition),
  ReleaseToFreelancer(ReleaseToFreelancerTransition),
  ApproveMilestone(ApproveMilestoneTransition),
  ReleaseRemaining(ReleaseRemainingTransition),
  SubmitWork(SubmitWorkTransition),
  Cancel(CancelTransition),
}

// Shared data snapshot for typestate transitions
#[derive(Clone, Debug)]
struct FlowData {
  billing_id: BillingId,
  employer_id: LocalUserId,
  freelancer_id: LocalUserId,
  amount: f64,
}

// ===== States as structs =====
#[derive(Debug)] struct QuotationPendingTS { data: FlowData }
#[derive(Debug)] struct PaidEscrowTS      { data: FlowData }
#[derive(Debug)] struct WorkSubmittedTS   { data: FlowData }
#[allow(dead_code)]
#[derive(Debug)] struct CompletedTS       { data: FlowData }
#[allow(dead_code)]
#[derive(Debug)] struct CancelledTS       { data: FlowData }
impl PaidEscrowTS {
  /// Build typestate from a Billing row; only Some if the status is PaidEscrow.
  fn try_from_billing(b: &Billing) -> Option<Self> {
    if b.status == BillingStatus::PaidEscrow {
      Some(PaidEscrowTS { data: into_ts(b) })
    } else {
      None
    }
  }

  /// Submit work (first submission) from PaidEscrow state.
  fn submit_work(self, desc: String, url: Option<String>) -> Result<SubmitWorkTransition, FastJobErrorType> {
    let form = form_submit_work(desc, url)?;
    Ok(SubmitWorkTransition { form })
  }
}
// ===== Allowed transitions (methods consume self) =====
impl QuotationPendingTS {
  pub fn approve_and_fund(self, wallet_id: WalletId) -> Result<FundEscrowTransition, FastJobErrorType> {
    let form = form_paid_escrow();
    let tx = FundEscrowTransition {
      form,
      wallet: WalletAction::PayToEscrow { wallet_id, amount: self.data.amount },
    };
    Ok(tx)
  }
  #[allow(dead_code)]
  pub fn cancel(self) -> CancelTransition {
    let form = form_cancelled();
    CancelTransition { form }
  }
}

impl WorkSubmittedTS {
  pub fn approve_milestone(self, amount: f64) -> Result<ApproveMilestoneTransition, FastJobErrorType> {
    if amount <= 0.0 { return Err(FastJobErrorType::InvalidField("milestone amount must be > 0".into())); }
    let form = form_touch_only();
    let wallet = WalletAction::ReleaseToFreelancer { user_id: self.data.freelancer_id, amount };
    Ok(ApproveMilestoneTransition { form, wallet })
  }
  pub fn approve_work(self) -> ReleaseToFreelancerTransition {
    let (form, wallet) = form_completed_and_wallet(self.data.freelancer_id, self.data.amount);
    ReleaseToFreelancerTransition { form, wallet }
  }
}

// Terminal states (no outbound transitions)
#[allow(dead_code)]
impl CompletedTS {}
#[allow(dead_code)]
impl CancelledTS {}

// Map Billing row → typestate
fn into_ts(b: &Billing) -> FlowData {
  FlowData {
    billing_id: b.id,
    employer_id: b.employer_id,
    freelancer_id: b.freelancer_id,
    amount: b.amount,
  }
}

// Helper: apply transitions (wallet + Crud update)
async fn apply_transition(pool: &mut DbPool<'_>, billing_id: BillingId, plan: Planned) -> FastJobResult<Billing> {
  // Wallet side-effects first (API expects &mut DbPool). If *_tx available, move inside txn.
  // Wallet side-effects first (API expects &mut DbPool).
  match &plan {
    // กันเงินเข้ากอง escrow จากกระเป๋านายจ้าง (employer wallet)
    Planned::FundEscrow(t) => {
      if let WalletAction::PayToEscrow { wallet_id, amount } = &t.wallet {
        Wallet::hold(pool, *wallet_id, *amount).await?;
      }
    }

    // ปล่อยเงินให้ฟรีแลนซ์ (จ่ายจากนายจ้าง -> ฟรีแลนซ์)
    Planned::ReleaseToFreelancer(t) => {
      if let WalletAction::ReleaseToFreelancer { user_id: freelancer_id, amount } = &t.wallet {
        // โหลด billing เพื่อรู้ employer_id
        let b = Billing::read(pool, billing_id).await?;
        let employer_wallet = Wallet::get_by_user(pool, b.employer_id).await?;
        let freelancer_wallet = Wallet::get_by_user(pool, *freelancer_id).await?;
        // ตัดจาก outstanding/total ของนายจ้าง
        Wallet::spend(pool, employer_wallet.id, *amount).await?;
        // เติมเข้ากระเป๋าฟรีแลนซ์
        Wallet::deposit(pool, freelancer_wallet.id, *amount).await?;
      }
    }

    // จ่ายบางงวด (milestone) ระหว่างงาน
    Planned::ApproveMilestone(t) => {
      if let WalletAction::ReleaseToFreelancer { user_id: freelancer_id, amount } = &t.wallet {
        let b = Billing::read(pool, billing_id).await?;
        let employer_wallet = Wallet::get_by_user(pool, b.employer_id).await?;
        let freelancer_wallet = Wallet::get_by_user(pool, *freelancer_id).await?;
        Wallet::spend(pool, employer_wallet.id, *amount).await?;
        Wallet::deposit(pool, freelancer_wallet.id, *amount).await?;
      }
    }

    // จ่ายส่วนที่เหลือเมื่อปิดงาน
    Planned::ReleaseRemaining(t) => {
      if let WalletAction::ReleaseToFreelancer { user_id: freelancer_id, amount } = &t.wallet {
        let b = Billing::read(pool, billing_id).await?;
        let employer_wallet = Wallet::get_by_user(pool, b.employer_id).await?;
        let freelancer_wallet = Wallet::get_by_user(pool, *freelancer_id).await?;
        Wallet::spend(pool, employer_wallet.id, *amount).await?;
        Wallet::deposit(pool, freelancer_wallet.id, *amount).await?;
      }
    }

    _ => {}
  }

  let form = match &plan {
    Planned::FundEscrow(t) => &t.form,
    Planned::ReleaseToFreelancer(t) => &t.form,
    Planned::ApproveMilestone(t) => &t.form,
    Planned::ReleaseRemaining(t) => &t.form,
    Planned::SubmitWork(t) => &t.form,
    Planned::Cancel(t) => &t.form,
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
pub struct LoadedWorkflow {
  pub billing: Billing,
  pub flow_data: FlowData,
  pub available_actions: Vec<&'static str>,
}
impl WorkFlowService {
  /// Load a billing row and prepare FlowData for typestate use.
  pub async fn load(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
  ) -> FastJobResult<LoadedWorkflow> {
    let b = Billing::read(pool, billing_id).await?;
    let data = into_ts(&b);
    let actions = match b.status {
      BillingStatus::QuotationPending => vec!["approveAndFund", "cancel"],
      BillingStatus::PaidEscrow => vec!["submitWork", "approveMilestone", "cancel"],
      BillingStatus::WorkSubmitted => vec!["approveWork", "approveMilestone", "cancel"],
      BillingStatus::Completed => vec![],
      BillingStatus::Cancelled => vec![],
      _ => vec![],
    };
    Ok(LoadedWorkflow {
      billing: b,
      flow_data: data,
      available_actions: actions,
    })
  }

  /// Cancel a billing from either side in any non-terminal state.
  pub async fn cancel_billing_any(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    actor_id: LocalUserId,
  ) -> FastJobResult<Billing> {
    let LoadedWorkflow { billing: b, .. } = Self::load(pool, billing_id).await?;
    if b.employer_id != actor_id && b.freelancer_id != actor_id {
      return Err(FastJobErrorType::InvalidField("Not a participant of this billing".into()).into());
    }
    if matches!(b.status, BillingStatus::Completed | BillingStatus::Cancelled) {
      return Err(FastJobErrorType::InvalidField("Billing already finalized".into()).into());
    }
    let plan = Planned::Cancel(CancelTransition { form: form_cancelled() });
    let updated = apply_transition(pool, billing_id, plan).await?;
    Ok(updated)
  }

  pub async fn validate_before_approve(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    employer_id: LocalUserId,
  ) -> FastJobResult<Billing> {
    let LoadedWorkflow { billing: b, .. } = Self::load(pool, billing_id).await?;
    if b.employer_id != employer_id {
      return Err(FastJobErrorType::InvalidField("Not the employer of this billing".into()).into());
    }
    if b.status == BillingStatus::Completed { return Ok(b); }
    if b.status != BillingStatus::WorkSubmitted {
      return Err(FastJobErrorType::InvalidField("Billing not ready to approve".into()).into());
    }
    Ok(b)
  }

  pub async fn create_billing_from_quotation(
    pool: &mut DbPool<'_>,
    freelancer_id: LocalUserId,
    data: ValidCreateInvoice,
  ) -> FastJobResult<Billing> {
    let inner = data.0;
    let description = build_detailed_description(&inner);
    let billing_form = BillingFromQuotation {
      employer_id: inner.employer_id,
      freelancer_id,
      post_id: inner.post_id,
      comment_id: inner.comment_id,
      description,
      amount: inner.amount,
      max_revisions: inner.revise_times,
      delivery_day: inner.delivery_day,
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
    let LoadedWorkflow { billing: b, flow_data: data, .. } = Self::load(pool, billing_id).await?;
    if b.employer_id != employer_id {
      return Err(FastJobErrorType::InvalidField("Not the employer of this billing".into()).into());
    }
    let plan = match b.status {
      BillingStatus::QuotationPending => {
        let tx = QuotationPendingTS { data }.approve_and_fund(wallet_id)?;
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
    let LoadedWorkflow { billing: b, .. } = Self::load(pool, billing_id).await?;
    if b.freelancer_id != freelancer_id {
      return Err(FastJobErrorType::InvalidField("Not the freelancer of this billing".into()).into());
    }
    // Pure typestate: build PaidEscrowTS from current billing or reject
    let ts = PaidEscrowTS::try_from_billing(&b)
      .ok_or_else(|| FastJobErrorType::InvalidField("Billing not in a submittable state".into()))?;

    let tx = ts.submit_work(work_description, deliverable_url)?;
    let updated = apply_transition(pool, billing_id, Planned::SubmitWork(tx)).await?;
    Ok(updated)
  }

  pub async fn approve_work(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    employer_id: LocalUserId,
  ) -> FastJobResult<Billing> {
    let LoadedWorkflow { billing: b, flow_data: data, .. } = Self::load(pool, billing_id).await?;
    if b.employer_id != employer_id {
      return Err(FastJobErrorType::InvalidField("Not the employer of this billing".into()).into());
    }
    let plan = match b.status {
      BillingStatus::WorkSubmitted => {
        let tx = WorkSubmittedTS { data }.approve_work();
        Planned::ReleaseToFreelancer(tx)
      }
      _ => return Err(FastJobErrorType::InvalidField("Billing not ready to approve".into()).into()),
    };
    let updated = apply_transition(pool, billing_id, plan).await?;
    Ok(updated)
  }

  /// Approve one milestone and release a partial payment without changing billing status.
  pub async fn approve_milestone(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    employer_id: LocalUserId,
    amount: f64,
  ) -> FastJobResult<Billing> {
    let LoadedWorkflow { billing: b, flow_data: data, .. } = Self::load(pool, billing_id).await?;
    if b.employer_id != employer_id {
      return Err(FastJobErrorType::InvalidField("Not the employer of this billing".into()).into());
    }
    let plan = if b.status == BillingStatus::WorkSubmitted {
      let tx = WorkSubmittedTS { data }.approve_milestone(amount)?;
      Planned::ApproveMilestone(tx)
    } else if b.paid_at.is_some() {
      // funded but not yet in WorkSubmitted state → still allow partial payout without status change
      if amount <= 0.0 { return Err(FastJobErrorType::InvalidField("milestone amount must be > 0".into()).into()); }
      let form = form_touch_only();
      let wallet = WalletAction::ReleaseToFreelancer { user_id: data.freelancer_id, amount };
      Planned::ApproveMilestone(ApproveMilestoneTransition { form, wallet })
    } else {
      return Err(FastJobErrorType::InvalidField("Escrow not funded".into()).into());
    };
    let updated = apply_transition(pool, billing_id, plan).await?;
    Ok(updated)
  }

  /// Approve work by releasing only the remaining amount (callers compute remaining themselves).
  pub async fn approve_work_with_remaining(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
    employer_id: LocalUserId,
    remaining_amount: f64,
  ) -> FastJobResult<Billing> {
    if remaining_amount <= 0.0 { return Err(FastJobErrorType::InvalidField("remaining_amount must be > 0".into()).into()); }
    let LoadedWorkflow { billing: b, flow_data: data, .. } = Self::load(pool, billing_id).await?;
    if b.employer_id != employer_id {
      return Err(FastJobErrorType::InvalidField("Not the employer of this billing".into()).into());
    }
    let form = BillingUpdateForm { status: Some(BillingStatus::Completed), updated_at: Some(Utc::now()), ..Default::default() };
    let wallet = WalletAction::ReleaseToFreelancer { user_id: data.freelancer_id, amount: remaining_amount };
    let plan = Planned::ReleaseRemaining(ReleaseRemainingTransition { form, wallet });
    let updated = apply_transition(pool, billing_id, plan).await?;
    Ok(updated)
  }
}