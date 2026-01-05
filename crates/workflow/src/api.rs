use app_108jobs_db_schema::newtypes::WorkflowId;

// Shared data snapshot for typestate transitions
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct FlowData {
  pub(crate) workflow_id: WorkflowId,
}

// ===== States as structs =====
#[derive(Debug)]
#[allow(dead_code)]
pub struct QuotationPendingReviewTS {
  pub(crate) data: FlowData,
}
#[derive(Debug)]
#[allow(dead_code)]
pub struct PaidEscrowTS {
  pub(crate) data: FlowData,
}
#[derive(Debug)]
#[allow(dead_code)]
pub struct WorkSubmittedTS {
  pub(crate) data: FlowData,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct CompletedTS {
  data: FlowData,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct CancelledTS {
  data: FlowData,
}
