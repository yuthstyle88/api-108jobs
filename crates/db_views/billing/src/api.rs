use chrono::NaiveDate;
use lemmy_db_schema::newtypes::{BillingId, ChatRoomId, Coin, CommentId, LocalUserId, PostId, WalletId, WorkflowId};
use lemmy_db_schema::source::billing::WorkStep;
use lemmy_db_schema_file::enums::BillingStatus;
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
    pub comment_id: CommentId,
    pub seq_number: i16,
    pub amount: Coin,
    pub proposal: String,
    pub project_name: String,
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
pub struct ApproveQuotation {
    pub seq_number: i16,
    pub billing_id: BillingId,
    pub wallet_id: WalletId,
    pub workflow_id: WorkflowId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Start or init a workflow for a post/sequence in a chat room.
pub struct StartWorkflow {
    pub post_id: PostId,
    pub seq_number: i16,
    pub room_id: ChatRoomId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Submit completed work (freelancer starts work).
pub struct SubmitStartWork {
    pub seq_number: i16,
    pub workflow_id: WorkflowId,
    pub work_description: String,
    pub deliverable_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Approve work and release payment (employer approves).
pub struct ApproveWork {
    pub seq_number: i16,
    pub workflow_id: WorkflowId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Request revision on a submitted work (employer requests changes from freelancer).
pub struct RequestRevision {
    pub seq_number: i16,
    pub workflow_id: WorkflowId,
    pub reason: Option<String>,
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
    pub delivery_timeframe_days: i32,
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
