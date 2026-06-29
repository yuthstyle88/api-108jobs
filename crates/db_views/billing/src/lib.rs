use app_108jobs_db::source::billing::Billing;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BillingView {
  pub billings: Vec<Billing>,
}

pub mod api;
pub mod impls;
pub mod validator;
pub use api::{
  ApproveQuotationForm,
  ApproveWorkForm,
  CancelJobForm,
  CreateInvoiceForm,
  CreateInvoiceResponse,
  GetBillingByRoomQuery,
  RequestRevisionForm,
  StartWorkflowForm,
  SubmitStartWorkForm,
  UpdateBudgetPlanInstallments,
  UpdateBudgetPlanInstallmentsResponse,
};
pub use validator::{
  ValidApproveQuotationRequest,
  ValidApproveWorkRequest,
  ValidCancelJobRequest,
  ValidCreateInvoiceRequest,
  ValidRequestRevisionRequest,
  ValidStartWorkflowRequest,
  ValidSubmitStartWorkRequest,
  ValidUpdateBudgetPlanInstallmentsRequest,
};
