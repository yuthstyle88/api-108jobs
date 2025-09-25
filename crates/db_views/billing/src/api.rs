use chrono::NaiveDate;
use lemmy_db_schema::newtypes::{BillingId, ChatRoomId, Coin, CommentId, LocalUserId, PostId, WalletId, WorkflowId};
use lemmy_db_schema::source::billing::WorkStep;
use lemmy_db_schema_file::enums::{BillingStatus, WorkFlowStatus};
use lemmy_utils::error::FastJobErrorType;
use serde::{Deserialize, Serialize};
use lemmy_db_schema::source::job_budget_plan::JobBudgetPlan;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Create invoice/quotation for job (freelancer creates detailed proposal).
pub struct CreateInvoiceForm {
    pub employer_id: LocalUserId,
    pub post_id: PostId,
    pub comment_id: Option<CommentId>,
    pub seq_number: i16,
    pub amount: Coin,
    pub proposal: String,
    pub project_name: String,
    pub status: BillingStatus,
    #[serde(default)]
    pub project_details: String,
    pub working_days: i32,
    #[serde(default)]
    pub deliverables: Vec<String>,
    pub note: Option<String>,
    #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
    pub starting_day: NaiveDate, // ISO date string (YYYY-MM-DD)
    #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
    pub delivery_day: NaiveDate, // ISO date string (YYYY-MM-DD)
    pub room_id: ChatRoomId,
}

/// Strongly-typed validated wrapper for CreateInvoice
#[derive(Debug, Clone)]
pub struct ValidCreateInvoice(pub CreateInvoiceForm);

impl TryFrom<CreateInvoiceForm> for ValidCreateInvoice {
    type Error = String;
    fn try_from(value: CreateInvoiceForm) -> Result<Self, Self::Error> {
        if value.amount <= 0 {
            return Err("Price must be positive".to_string());
        }
        if value.seq_number <= 0 {
            return Err("Invalid sequent number".to_string());
        }
        Ok(ValidCreateInvoice(value))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Approve quotation and convert to order (employer approves freelancer's quotation).
pub struct ApproveQuotationForm {
    pub seq_number: i16,
    pub billing_id: BillingId,
    pub wallet_id: WalletId,
    pub workflow_id: WorkflowId,
}

#[derive(Debug, Clone)]
pub struct ValidApproveQuotation(pub ApproveQuotationForm);

impl TryFrom<ApproveQuotationForm> for ValidApproveQuotation {
    type Error = String;
    fn try_from(value: ApproveQuotationForm) -> Result<Self, Self::Error> {
        if value.seq_number <= 0 {
            return Err("Invalid sequent number".into());
        }
        Ok(ValidApproveQuotation(value))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Start or init a workflow for a post/sequence in a chat room.
pub struct StartWorkflowForm {
    pub post_id: PostId,
    pub seq_number: i16,
    pub room_id: ChatRoomId,
}

#[derive(Debug, Clone)]
pub struct ValidStartWorkflow(pub StartWorkflowForm);

impl TryFrom<StartWorkflowForm> for ValidStartWorkflow {
    type Error = String;
    fn try_from(value: StartWorkflowForm) -> Result<Self, Self::Error> {
        if value.seq_number <= 0 {
            return Err("Invalid sequent number".into());
        }
        Ok(ValidStartWorkflow(value))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Submit completed work (freelancer starts work).
pub struct SubmitStartWorkForm {
    pub seq_number: i16,
    pub workflow_id: WorkflowId,
    pub work_description: String,
    pub deliverable_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValidSubmitStartWork(pub SubmitStartWorkForm);

impl TryFrom<SubmitStartWorkForm> for ValidSubmitStartWork {
    type Error = String;
    fn try_from(value: SubmitStartWorkForm) -> Result<Self, Self::Error> {
        if value.seq_number <= 0 {
            return Err("Invalid sequent number".into());
        }
        if value.work_description.trim().is_empty() {
            return Err("Work description is required".into());
        }
        Ok(ValidSubmitStartWork(value))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Approve work and release payment (employer approves).
pub struct ApproveWorkForm {
    pub seq_number: i16,
    pub workflow_id: WorkflowId,
    pub room_id: ChatRoomId,
}

#[derive(Debug, Clone)]
pub struct ValidApproveWork(pub ApproveWorkForm);

impl TryFrom<ApproveWorkForm> for ValidApproveWork {
    type Error = String;
    fn try_from(value: ApproveWorkForm) -> Result<Self, Self::Error> {
        if value.seq_number <= 0 {
            return Err("Invalid sequent number".into());
        }
        Ok(ValidApproveWork(value))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Request revision on a submitted work (employer requests changes from freelancer).
pub struct RequestRevisionForm {
    pub seq_number: i16,
    pub workflow_id: WorkflowId,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValidRequestRevision(pub RequestRevisionForm);

impl TryFrom<RequestRevisionForm> for ValidRequestRevision {
    type Error = String;
    fn try_from(value: RequestRevisionForm) -> Result<Self, Self::Error> {
        if value.seq_number <= 0 {
            return Err("Invalid sequent number".into());
        }
        Ok(ValidRequestRevision(value))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Response for creating an invoice.
pub struct CreateInvoiceResponse {
    pub billing_id: BillingId,
    pub issuer_id: LocalUserId,
    pub recipient_id: LocalUserId,
    pub post_id: PostId,
    pub amount: Coin,
    pub status: BillingStatus,
    pub created_at: String,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Response for billing operations.
pub struct BillingOperationResponse {
    pub billing_id: BillingId,
    pub status: BillingStatus,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Update the entire installments array for a given post.
pub struct UpdateBudgetPlanInstallments {
    pub post_id: PostId,
    pub installments: Vec<WorkStep>,
}

/// Strongly-typed validated wrapper for CreateInvoice
#[derive(Debug, Clone)]
pub struct ValidUpdateBudgetPlanInstallments(pub UpdateBudgetPlanInstallments);

impl TryFrom<UpdateBudgetPlanInstallments> for ValidUpdateBudgetPlanInstallments {
    type Error = FastJobErrorType;
    fn try_from(value: UpdateBudgetPlanInstallments) -> Result<Self, Self::Error> {
        let items = value.installments.clone();

        // Basic validation: idx positive, unique; amount >= 0; status is "paid" or "unpaid"
        for it in &items {
            if it.amount < 0 {
                return Err(FastJobErrorType::NegativeAmount.into());
            }
        }
        Ok(ValidUpdateBudgetPlanInstallments(value))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Response for creating an invoice.
pub struct UpdateBudgetPlanInstallmentsResponse {
    pub budget_plan: JobBudgetPlan,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Cancel a workflow job
pub struct CancelJobForm {
    pub seq_number: i16,
    pub workflow_id: WorkflowId,
    pub reason: Option<String>,
    pub current_status: WorkFlowStatus,
}

#[derive(Debug, Clone)]
pub struct ValidCancelJob(pub CancelJobForm);

impl TryFrom<CancelJobForm> for ValidCancelJob {
    type Error = String;
    fn try_from(value: CancelJobForm) -> Result<Self, Self::Error> {
        if value.seq_number <= 0 {
            return Err("Invalid sequent number".into());
        }
        Ok(ValidCancelJob(value))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBillingByCommentQuery {
    pub comment_id: CommentId,
}
