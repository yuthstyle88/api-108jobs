use lemmy_db_schema::newtypes::WorkflowId;

use crate::{CancelTransition, FundEscrowTransition, ReleaseRemainingTransition, ReleaseToFreelancerTransition, SubmitWorkTransition};

// Planner enum: unifies all transition variants for the DB/apply layer
pub enum Planned {
  FundEscrow(FundEscrowTransition),
  ReleaseToFreelancer(ReleaseToFreelancerTransition),
  ReleaseRemaining(ReleaseRemainingTransition),
  SubmitWork(SubmitWorkTransition),
  Cancel(CancelTransition),
}

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