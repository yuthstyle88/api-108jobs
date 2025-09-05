use lemmy_db_schema::newtypes::WorkflowId;

// Shared data snapshot for typestate transitions
#[derive(Clone, Debug)]
pub struct FlowData {
  pub(crate) workflow_id: WorkflowId,
}

// ===== States as structs =====
#[derive(Debug)] pub struct QuotationPendingTS { pub(crate) data: FlowData }
#[derive(Debug)] pub struct PaidEscrowTS      { pub(crate) data: FlowData }
#[derive(Debug)] pub struct WorkSubmittedTS   { pub(crate) data: FlowData }
#[allow(dead_code)]
#[derive(Debug)] pub struct CompletedTS       { data: FlowData }
#[allow(dead_code)]
#[derive(Debug)] pub struct CancelledTS       { data: FlowData }