use serde::{Deserialize, Serialize};
use lemmy_db_schema::newtypes::WorkflowId;
use lemmy_db_schema::source::workflow::WorkflowUpdateForm;
use lemmy_db_schema_file::enums::WorkFlowStatus;

mod api;
mod impls;

pub use crate::impls::WorkflowService;

/// Workflow/command operations for billing lifecycle (create, approve, submit, revise, complete).
// ===== Typestate State Machine (structs-only) =====
// Each state is a distinct struct; allowed transitions are methods that
// consume the current state and return the next state's struct + a domain transition payload.

// Domain transitions used by apply_transition()
struct FundEscrowTransition { pub form: WorkflowUpdateForm }
struct ReleaseToFreelancerTransition { pub form: WorkflowUpdateForm }
struct ReleaseRemainingTransition { pub form: WorkflowUpdateForm }
struct SubmitWorkTransition { pub form: WorkflowUpdateForm }
struct CancelTransition { pub form: WorkflowUpdateForm }
// NOTE: No rollback (prev) transitions are supported. To restart, cancel this billing and open a new one.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct WorkFlowOperationResponse{
  pub workflow_id: WorkflowId,
  pub status: WorkFlowStatus,
  pub success: bool,
}