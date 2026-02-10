use app_108jobs_db_schema::source::billing::Billing;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BillingView {
  pub billings: Vec<Billing>,
}

pub mod impls;
pub mod api;
pub mod validator;
pub use validator::{
  ValidApproveQuotationRequest, ValidApproveWorkRequest, ValidCancelJobRequest, ValidCreateInvoiceRequest,
  ValidRequestRevisionRequest, ValidStartWorkflowRequest, ValidSubmitStartWorkRequest, ValidUpdateBudgetPlanInstallmentsRequest,
};
pub use api::{
  ApproveQuotationForm, ApproveWorkForm, CancelJobForm, CreateInvoiceForm, CreateInvoiceResponse,
  GetBillingByRoomQuery, RequestRevisionForm, StartWorkflowForm, SubmitStartWorkForm,
  UpdateBudgetPlanInstallments, UpdateBudgetPlanInstallmentsResponse,
};