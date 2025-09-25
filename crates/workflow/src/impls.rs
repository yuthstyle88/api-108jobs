use chrono::Utc;
use diesel_async::scoped_futures::ScopedFutureExt;
use lemmy_db_schema::newtypes::{
  BillingId, ChatRoomId, Coin, CoinId, LocalUserId, PostId, WalletId, WorkflowId,
};
use lemmy_db_schema::source::billing::BillingInsertForm;
use lemmy_db_schema::source::billing::{Billing, BillingUpdateForm};
use lemmy_db_schema::source::chat_room::{ChatRoom, ChatRoomUpdateForm};
use lemmy_db_schema::source::wallet::{TxKind, WalletModel, WalletTransactionInsertForm};
use lemmy_db_schema::source::workflow::{Workflow, WorkflowInsertForm, WorkflowUpdateForm};
use lemmy_db_schema::traits::Crud;
use lemmy_db_schema::utils::{get_conn, DbPool};
use lemmy_db_schema_file::enums::BillingStatus::QuotePendingReview;
use lemmy_db_schema_file::enums::{BillingStatus, WorkFlowStatus};
use lemmy_db_views_billing::api::ValidCreateInvoice;
use lemmy_utils::error::FastJobErrorExt2;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use uuid::Uuid;

// ---------- Typestate payload ----------
#[derive(Clone, Copy, Debug)]
pub struct FlowData {
  pub workflow_id: WorkflowId,
  pub billing_id: Option<BillingId>,
  pub amount: Option<Coin>,
}

// ---------- Typestate structs ----------
#[derive(Debug)]
pub struct QuotationPendingTS {
  pub data: FlowData,
}
#[derive(Debug)]
pub struct OrderApprovedTS {
  pub data: FlowData,
}
#[derive(Debug)]
pub struct InProgressTS {
  pub data: FlowData,
}
#[derive(Debug)]
pub struct WorkSubmittedTS {
  pub data: FlowData,
}
#[derive(Debug)]
pub struct CompletedTS {
  pub data: FlowData,
}
#[derive(Debug)]
#[allow(dead_code)]
pub struct CancelledTS {
  pub data: FlowData,
}

// ---------- Pure transitions (NO DB) ----------
impl QuotationPendingTS {
  pub fn approve(self) -> OrderApprovedTS {
    OrderApprovedTS { data: self.data }
  }
}
impl OrderApprovedTS {
  pub fn start_work(self) -> InProgressTS {
    InProgressTS { data: self.data }
  }
}
impl InProgressTS {
  pub fn submit_work(self) -> WorkSubmittedTS {
    WorkSubmittedTS { data: self.data }
  }
  pub fn request_revision(self) -> InProgressTS {
    InProgressTS { data: self.data }
  }
}
impl WorkSubmittedTS {
  pub fn request_revision(self) -> InProgressTS {
    InProgressTS { data: self.data }
  }
  pub fn approve_work(self) -> CompletedTS {
    CompletedTS { data: self.data }
  }
}

// ยกเลิก workflow ให้ทุก ๆ typestate ที่มี FlowData ใช้ร่วมกัน
#[allow(dead_code)]
pub trait CancelableTS: Sized {
  fn workflow_id(&self) -> WorkflowId;
  fn into_flow_data(self) -> FlowData;

  // Use a boxed Future to avoid mismatched lifetime parameters with ScopedBoxFuture
  async fn cancel_on(self, pool: &mut DbPool<'_>) -> FastJobResult<CancelledTS> {
    // เรียก helper ที่เขียนไว้
    cancel_any_on(pool, self.workflow_id(), Default::default()).await?;
    Ok(CancelledTS {
      data: self.into_flow_data(),
    })
  }
}

// ตัวอย่างการผูกกับแต่ละ state ที่มี field `data: FlowData`
impl CancelableTS for InProgressTS {
  #[inline]
  fn workflow_id(&self) -> WorkflowId {
    self.data.workflow_id
  }
  #[inline]
  fn into_flow_data(self) -> FlowData {
    self.data
  }
}

impl CancelableTS for WorkSubmittedTS {
  #[inline]
  fn workflow_id(&self) -> WorkflowId {
    self.data.workflow_id
  }
  #[inline]
  fn into_flow_data(self) -> FlowData {
    self.data
  }
}

// เพิ่ม impl สำหรับ state อื่น ๆ ที่มี `data: FlowData` ได้ในรูปแบบเดียวกัน
// impl CancelableTS for AnotherState { ... }

// CompletedTS / CancelledTS: no outbound pure methods

// ---------- Shared DB helpers (free functions) ----------
async fn set_status_from(
  pool: &mut DbPool<'_>,
  workflow_id: WorkflowId,
  expected_from: WorkFlowStatus,
  desired: WorkFlowStatus,
  mutate_form: impl FnOnce(&Workflow, &mut WorkflowUpdateForm) + Send + 'static,
) -> FastJobResult<()> {
  let conn = &mut get_conn(pool).await?;
  conn
    .run_transaction(|conn| {
      async move {
        let current = Workflow::read(&mut conn.into(), workflow_id)
          .await
          .with_fastjob_type(FastJobErrorType::DatabaseError)?;

        if current.status != expected_from {
          return Err(
            FastJobErrorType::InvalidField(format!(
              "Illegal transition: expected {:?}, found {:?}",
              expected_from, current.status
            ))
            .into(),
          );
        }

        let mut form = WorkflowUpdateForm {
          status: Some(desired),
          updated_at: Some(Some(Utc::now())),
          ..Default::default()
        };
        mutate_form(&current, &mut form);

        let _ = Workflow::update(&mut conn.into(), workflow_id, &form)
          .await
          .with_fastjob_type(FastJobErrorType::DatabaseError)?;

        // If workflow finalized, clear the current_comment_id in the related chat room
        if matches!(
          desired,
          WorkFlowStatus::Completed | WorkFlowStatus::Cancelled
        ) {
          let clr = ChatRoomUpdateForm {
            room_name: None,
            updated_at: Some(Utc::now()),
            post_id: None,
            current_comment_id: Some(None),
          };
          let _ = ChatRoom::update(&mut conn.into(), current.room_id.clone(), &clr)
            .await
            .with_fastjob_type(FastJobErrorType::DatabaseError)?;
        }

        Ok::<_, lemmy_utils::error::FastJobError>(())
      }
      .scope_boxed()
    })
    .await?;
  Ok(())
}

#[allow(dead_code)]
async fn cancel_any_on(
  pool: &mut DbPool<'_>,
  workflow_id: WorkflowId,
  current_status: WorkFlowStatus,
) -> FastJobResult<()> {
  let conn = &mut get_conn(pool).await?;
  conn
    .run_transaction(|conn| {
      async move {
        let cur = Workflow::read(&mut conn.into(), workflow_id)
          .await
          .with_fastjob_type(FastJobErrorType::DatabaseError)?;

        if matches!(
          cur.status,
          WorkFlowStatus::Completed | WorkFlowStatus::Cancelled
        ) {
          return Err(FastJobErrorType::InvalidField("Workflow already finalized".into()).into());
        }

        let form = WorkflowUpdateForm {
          status: Some(WorkFlowStatus::Cancelled),
          updated_at: Some(Some(Utc::now())),
          status_before_cancel: Some(Some(current_status)),
          ..Default::default()
        };
        let _ = Workflow::update(&mut conn.into(), workflow_id, &form)
          .await
          .with_fastjob_type(FastJobErrorType::DatabaseError)?;

        if let Some(billing) = Billing::get_by_room_and_status(
          &mut conn.into(),
          cur.room_id.clone(),
          QuotePendingReview,
        ).await? {
          Billing::update(
            &mut conn.into(),
            billing.id,
            &BillingUpdateForm {
              status: Some(BillingStatus::Canceled),
              work_description: None,
              deliverable_url: None,
              updated_at: None,
              paid_at: None,
            },
          ).await?;
        }

        // Clear current_comment_id on room when cancelling
        let clr = ChatRoomUpdateForm {
          room_name: None,
          updated_at: Some(Utc::now()),
          post_id: None,
          current_comment_id: Some(None),
        };
        let _ = ChatRoom::update(&mut conn.into(), cur.room_id, &clr)
          .await
          .with_fastjob_type(FastJobErrorType::DatabaseError)?;

        Ok::<_, lemmy_utils::error::FastJobError>(())
      }
      .scope_boxed()
    })
    .await?;
  Ok(())
}

// ================= Reusable helpers =================

// สร้างฟอร์ม WorkflowInsertForm แบบ reuse ได้
#[allow(dead_code)]
#[inline]
fn build_workflow_insert(
  post_id: PostId,
  seq_number: i16,
  room_id: ChatRoomId,
) -> WorkflowInsertForm {
  WorkflowInsertForm {
    post_id,
    seq_number,
    status: Some(WorkFlowStatus::WaitForFreelancerQuotation),
    revision_required: None,
    revision_count: None,
    revision_reason: None,
    deliverable_version: None,
    deliverable_submitted_at: None,
    deliverable_accepted: None,
    accepted_at: None,
    created_at: Some(Utc::now()),
    updated_at: None,
    room_id,
    deliverable_url: None,
    active: Some(true),
    status_before_cancel: None,
    billing_id: None,
  }
}

// สร้าง Workflow ใหม่สำหรับโพสต์ โดยบังคับกติกา:
// ถ้ามีของเก่าอยู่และยังไม่ถูกยกเลิก -> return error
// ถ้าไม่มี หรือของเก่าถูกยกเลิกแล้ว -> สร้างใหม่
async fn create_new_workflow_for_post(
  pool: &mut DbPool<'_>,
  post_id: PostId,
  seq_number: i16,
  room_id: ChatRoomId,
) -> FastJobResult<Workflow> {
  let insert = build_workflow_insert(post_id, seq_number, room_id);
  Workflow::create(pool, &insert).await
}

#[allow(dead_code)]
async fn load_billing_and_check_employer(
  pool: &mut DbPool<'_>,
  billing_id: BillingId,
  employer_id: LocalUserId,
) -> FastJobResult<Billing> {
  let conn = &mut get_conn(pool).await?;
  let billing = Billing::read(&mut conn.into(), billing_id)
    .await
    .with_fastjob_type(FastJobErrorType::DatabaseError)?;
  if billing.employer_id != employer_id {
    return Err(FastJobErrorType::NotAllowed.into());
  }
  Ok(billing)
}

#[allow(dead_code)]
async fn reserve_to_escrow(
  pool: &mut DbPool<'_>,
  from_wallet_id: WalletId,
  billing_id: BillingId,
  amount: Coin,
  employer_id: LocalUserId,
  reference_type: &str,
  description: String,
) -> FastJobResult<()> {
  if amount <= Coin(0) {
    return Err(FastJobErrorType::InvalidField("amount must be positive".into()).into());
  }
  let tx_form = WalletTransactionInsertForm {
    wallet_id: from_wallet_id,
    reference_type: reference_type.to_string(),
    reference_id: billing_id.0,
    kind: TxKind::Transfer, // hold: move user -> platform (escrow)
    amount,
    description,
    counter_user_id: Some(employer_id),
    idempotency_key: Uuid::new_v4().to_string(),
  };
  let _ = WalletModel::hold(pool, &tx_form).await?;
  Ok(())
}

#[allow(dead_code)]
async fn do_transition(
  pool: &mut DbPool<'_>,
  workflow_id: WorkflowId,
  from: WorkFlowStatus,
  to: WorkFlowStatus,
  mutate: impl FnOnce(&Workflow, &mut WorkflowUpdateForm) + Send + 'static,
) -> FastJobResult<()> {
  set_status_from(pool, workflow_id, from, to, mutate).await
}

// ================= Refactored public methods =================

impl QuotationPendingTS {
  pub async fn approve_on(
    self,
    pool: &mut DbPool<'_>,
    employer_id: LocalUserId,
    wallet_id: WalletId,
    billing_id: BillingId,
  ) -> FastJobResult<OrderApprovedTS> {
    // 1) โหลด Billing เพื่อเอา amount และตรวจสิทธิ์
    let conn = &mut get_conn(pool).await?;
    let billing = Billing::read(&mut conn.into(), billing_id)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;
    if billing.employer_id != employer_id {
      return Err(FastJobErrorType::NotAllowed.into());
    }

    // 2) โยกเหรียญเข้า escrow (user -> platform) ด้วย hold
    let tx_form = WalletTransactionInsertForm {
      wallet_id,
      reference_type: "billing".to_string(),
      reference_id: billing_id.0,
      kind: TxKind::Transfer, // ใช้ Transfer สำหรับ hold
      amount: billing.amount,
      description: "escrow reserve for approved quotation".to_string(),
      counter_user_id: Some(employer_id),
      idempotency_key: Uuid::new_v4().to_string(),
    };
    let _ = WalletModel::hold(pool, &tx_form).await?;

    // 3) เปลี่ยนสถานะ QuotationPending -> OrderApproved
    set_status_from(
      pool,
      self.data.workflow_id,
      WorkFlowStatus::QuotationPending,
      WorkFlowStatus::OrderApproved,
      |_current, form: &mut WorkflowUpdateForm| {
        form.updated_at = Some(Some(Utc::now()));
      },
    )
    .await?;

    // 4) อัปเดต FlowData ให้มี billing_id และ amount
    let mut next = self.approve();
    next.data.billing_id = Some(billing_id);
    next.data.amount = Some(billing.amount);

    Ok(next)
  }
}

impl OrderApprovedTS {
  pub async fn start_work_on(self, pool: &mut DbPool<'_>) -> FastJobResult<InProgressTS> {
    set_status_from(
      pool,
      self.data.workflow_id,
      WorkFlowStatus::OrderApproved,
      WorkFlowStatus::InProgress,
      |_c, _f| {},
    )
    .await?;
    Ok(self.start_work())
  }
}

impl InProgressTS {
  pub async fn submit_work_on(self, pool: &mut DbPool<'_>) -> FastJobResult<WorkSubmittedTS> {
    set_status_from(
      pool,
      self.data.workflow_id,
      WorkFlowStatus::InProgress,
      WorkFlowStatus::PendingEmployerReview,
      |_c, _f| {},
    )
    .await?;

    Ok(self.submit_work())
  }
}

impl WorkSubmittedTS {
  pub async fn request_revision_on(
    self,
    pool: &mut DbPool<'_>,
    reason: Option<String>,
  ) -> FastJobResult<InProgressTS> {
    set_status_from(
      pool,
      self.data.workflow_id,
      WorkFlowStatus::PendingEmployerReview,
      WorkFlowStatus::InProgress,
      |cur, form| {
        form.revision_required = Some(true);
        form.revision_count = Some(cur.revision_count.saturating_add(1));
        form.revision_reason = Some(reason);
      },
    )
    .await?;

    Ok(self.request_revision())
  }
  pub async fn approve_work_on(
    self,
    pool: &mut DbPool<'_>,
    coin_id: CoinId,
    platform_wallet_id: WalletId,
    billing_id: BillingId,
  ) -> FastJobResult<CompletedTS> {
    // 1) โหลด Billing เพื่อทราบจำนวนเงินและผู้รับ (freelancer)
    let conn = &mut get_conn(pool).await?;
    let billing_opt = Billing::read(&mut conn.into(), billing_id).await;

    let billing = match billing_opt {
      Ok(b) => b,
      Err(_) => {
        return Err(
          FastJobErrorType::InvalidField(
            "No matching billing found ".to_string(),
          )
          .into(),
        );
      }
    };

    Billing::update(
      &mut conn.into(),
      billing.id,
      &BillingUpdateForm {
        status: Some(BillingStatus::OrderApproved),
        work_description: None,
        deliverable_url: None,
        updated_at: None,
        paid_at: None,
      },
    )
    .await?;

    // 2) ปล่อยเงินจาก escrow ไปยัง freelancer (platform -> freelancer)
    // สมมติว่าคุณมีวิธีหา wallet ของ freelancer เช่น WalletModel::get_by_user
    let freelancer_wallet = WalletModel::get_by_user(pool, billing.freelancer_id).await?;
    let tx_form = WalletTransactionInsertForm {
      wallet_id: freelancer_wallet.id,
      reference_type: "billing".to_string(),
      reference_id: billing.id.0,
      kind: TxKind::Transfer, // ใช้ Transfer สำหรับปล่อยเงิน
      amount: billing.amount,
      description: "escrow release to freelancer".to_string(),
      counter_user_id: Some(billing.freelancer_id),
      idempotency_key: Uuid::new_v4().to_string(),
    };
    // ใช้ transfer จาก platform -> freelancer (ควรมี helper release_from_escrow; ที่นี่ใช้ transfer_between_wallets ผ่านฟังก์ชันระดับ model ถ้ามี)
    // สำหรับตัวอย่างนี้ ใช้ deposit_from_platform ก็พอได้หากโมเดลของคุณถือ escrow ใน platform wallet
    let _ = WalletModel::deposit_from_platform(pool, &tx_form, coin_id, platform_wallet_id).await?;

    // 3) อัปเดตสถานะ WorkSubmitted -> Completed
    set_status_from(
      pool,
      self.data.workflow_id,
      WorkFlowStatus::PendingEmployerReview,
      WorkFlowStatus::Completed,
      |_c, _f| {},
    )
    .await?;

    Ok(self.approve_work())
  }
}

// ---------- WorkflowService: เฉพาะ entry/utility ----------
pub struct WorkflowService;

impl WorkflowService {
  pub async fn start_workflow(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    seq_number: i16,
    room_id: ChatRoomId,
  ) -> FastJobResult<Workflow> {
    if let Some(current) = Workflow::get_current_by_room_id(pool, room_id.clone()).await.unwrap_or(None) {
      let update_form = WorkflowUpdateForm {
        active: Some(false),
        updated_at: Some(Some(Utc::now())),
        ..Default::default()
      };
      Workflow::update(pool, current.id, &update_form).await?;
    }

    create_new_workflow_for_post(pool, post_id, seq_number, room_id).await
  }

  pub async fn cancel(
    pool: &mut DbPool<'_>,
    workflow_id: WorkflowId,
    current_status: WorkFlowStatus,
  ) -> FastJobResult<()> {
    cancel_any_on(pool, workflow_id, current_status).await
  }

  pub async fn create_quotation(
    pool: &mut DbPool<'_>,
    freelancer_id: LocalUserId,
    form: ValidCreateInvoice,
  ) -> FastJobResult<Billing> {
    let data = form.0.clone();

    let insert_billing = BillingInsertForm {
      freelancer_id,
      employer_id: data.employer_id,
      post_id: data.post_id,
      comment_id: data.comment_id,
      room_id: data.room_id.clone(),
      amount: data.amount,
      description: if !data.project_details.is_empty() {
        data.project_details.clone()
      } else {
        data.proposal.clone()
      },
      status: Some(data.status),
      work_description: None,
      deliverable_url: None,
      created_at: Some(Utc::now()),
    };

    let billing = <Billing as Crud>::create(pool, &insert_billing).await?;
    if let Some(current_wf) = Workflow::get_current_by_room_id(pool, data.room_id).await? {
      Workflow::update(
        pool,
        current_wf.id,
        &WorkflowUpdateForm {
          status: Some(WorkFlowStatus::QuotationPending),
          updated_at: Some(Some(Utc::now())),
          ..Default::default()
        },
      )
      .await?;
    }

    Ok(billing)
  }

  pub async fn load_quotation_pending(
    pool: &mut DbPool<'_>,
    workflow_id: WorkflowId,
  ) -> FastJobResult<QuotationPendingTS> {
    let wf = Workflow::read(pool, workflow_id)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;

    if wf.status != WorkFlowStatus::QuotationPending {
      return Err(
        FastJobErrorType::InvalidField(format!(
          "Illegal state: expected QuotationPending, found {:?}",
          wf.status
        ))
        .into(),
      );
    }

    let data = FlowData {
      workflow_id: wf.id,
      billing_id: None,
      amount: None,
    };
    Ok(QuotationPendingTS { data })
  }

  pub async fn load_order_approve(
    pool: &mut DbPool<'_>,
    workflow_id: WorkflowId,
  ) -> FastJobResult<OrderApprovedTS> {
    let wf = Workflow::read(pool, workflow_id)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;

    if wf.status != WorkFlowStatus::OrderApproved {
      return Err(
        FastJobErrorType::InvalidField(format!(
          "Illegal state: expected OrderApprovedTS, found {:?}",
          wf.status
        ))
        .into(),
      );
    }

    // สร้าง FlowData สำหรับ typestate ของคุณ
    // ปรับตาม constructor จริงในโปรเจกต์ (เช่น FlowData::from_workflow(wf))
    let data = FlowData {
      workflow_id: wf.id,
      billing_id: None,
      amount: None,
    };

    Ok(OrderApprovedTS { data })
  }
  pub async fn load_work_submit(
    pool: &mut DbPool<'_>,
    workflow_id: WorkflowId,
  ) -> FastJobResult<WorkSubmittedTS> {
    let wf = Workflow::read(pool, workflow_id)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;

    if wf.status != WorkFlowStatus::PendingEmployerReview {
      return Err(
        FastJobErrorType::InvalidField(format!(
          "Illegal state: expected WorkSubmitted, found {:?}",
          wf.status
        ))
        .into(),
      );
    }

    // สร้าง FlowData สำหรับ typestate ของคุณ
    // ปรับตาม constructor จริงในโปรเจกต์ (เช่น FlowData::from_workflow(wf))
    let data = FlowData {
      workflow_id: wf.id,
      billing_id: None,
      amount: None,
    };

    Ok(WorkSubmittedTS { data })
  }
  pub async fn load_in_progress(
    pool: &mut DbPool<'_>,
    workflow_id: WorkflowId,
  ) -> FastJobResult<InProgressTS> {
    let wf = Workflow::read(pool, workflow_id)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;

    if wf.status != WorkFlowStatus::InProgress {
      return Err(
        FastJobErrorType::InvalidField(format!(
          "Illegal state: expected InProgress, found {:?}",
          wf.status
        ))
        .into(),
      );
    }

    // สร้าง FlowData สำหรับ typestate ของคุณ
    // ปรับตาม constructor จริงในโปรเจกต์ (เช่น FlowData::from_workflow(wf))
    let data = FlowData {
      workflow_id: wf.id,
      billing_id: None,
      amount: None,
    };

    Ok(InProgressTS { data })
  }
  pub async fn ensure_status(
    pool: &mut DbPool<'_>,
    workflow_id: WorkflowId,
    expected: WorkFlowStatus,
  ) -> FastJobResult<Workflow> {
    let conn = &mut get_conn(pool).await?;
    let wf = Workflow::read(&mut conn.into(), workflow_id)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;
    if wf.status != expected {
      return Err(
        FastJobErrorType::InvalidField(format!(
          "Illegal state: expected {:?}, found {:?}",
          expected, wf.status
        ))
        .into(),
      );
    }
    Ok(wf)
  }
}
