//! Authorization policy for escrow workflow transitions.
//!
//! These are **pure** functions (no IO) so the authorization matrix can be
//! unit-tested without a database. The HTTP handlers in [`super::workflow`] load
//! the `Workflow`/`Billing`/`Post` rows and pass the relevant ids in here.
//!
//! Matrix (locked 2026-06-28): create-invoice/start-work/submit-work →
//! freelancer; approve-quotation/request-revision/approve-work → employer;
//! cancel-job → either party; no admin override. Wrong party → `NotFound`
//! (do not reveal the workflow's existence to non-parties).
//!
//! See `docs/superpowers/specs/2026-06-28-workflow-escrow-authorization-design.md`.

use app_108jobs_core::error::{FastJobErrorType, FastJobResult};
use app_108jobs_db_schema::newtypes::{LocalUserId, PersonId};

/// Which side of a job the caller is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowRole {
  Employer,
  Freelancer,
}

/// The caller's role given a billing's two parties, or `None` if the caller is
/// neither the employer nor the freelancer.
pub fn caller_role(
  caller: LocalUserId,
  employer_id: LocalUserId,
  freelancer_id: LocalUserId,
) -> Option<WorkflowRole> {
  if caller == employer_id {
    Some(WorkflowRole::Employer)
  } else if caller == freelancer_id {
    Some(WorkflowRole::Freelancer)
  } else {
    None
  }
}

/// `Ok(())` iff `caller` holds `required` for this billing; otherwise
/// `NotFound`.
pub fn require_role(
  required: WorkflowRole,
  caller: LocalUserId,
  employer_id: LocalUserId,
  freelancer_id: LocalUserId,
) -> FastJobResult<()> {
  match caller_role(caller, employer_id, freelancer_id) {
    Some(role) if role == required => Ok(()),
    _ => Err(FastJobErrorType::NotFound.into()),
  }
}

/// `Ok(())` iff `caller` is either party (employer or freelancer); otherwise
/// `NotFound`. Used by `cancel-job`.
pub fn require_any_party(
  caller: LocalUserId,
  employer_id: LocalUserId,
  freelancer_id: LocalUserId,
) -> FastJobResult<()> {
  match caller_role(caller, employer_id, freelancer_id) {
    Some(_) => Ok(()),
    None => Err(FastJobErrorType::NotFound.into()),
  }
}

/// `Ok(())` iff the caller is the post creator (the employer), compared by
/// person id; otherwise `NotFound`. Used for pre-billing transitions
/// (`start-workflow`, `budget-plan`) where no `Billing` row exists yet.
pub fn require_post_creator(caller_person: PersonId, post_creator: PersonId) -> FastJobResult<()> {
  if caller_person == post_creator {
    Ok(())
  } else {
    Err(FastJobErrorType::NotFound.into())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn ids() -> (LocalUserId, LocalUserId, LocalUserId) {
    // employer, freelancer, stranger
    (LocalUserId(1), LocalUserId(2), LocalUserId(3))
  }

  #[test]
  fn caller_role_identifies_each_party() {
    let (employer, freelancer, stranger) = ids();
    assert_eq!(
      caller_role(employer, employer, freelancer),
      Some(WorkflowRole::Employer)
    );
    assert_eq!(
      caller_role(freelancer, employer, freelancer),
      Some(WorkflowRole::Freelancer)
    );
    assert_eq!(caller_role(stranger, employer, freelancer), None);
  }

  #[test]
  fn require_role_employer_only_allows_employer() {
    let (employer, freelancer, stranger) = ids();
    assert!(require_role(WorkflowRole::Employer, employer, employer, freelancer).is_ok());
    assert!(require_role(WorkflowRole::Employer, freelancer, employer, freelancer).is_err());
    assert!(require_role(WorkflowRole::Employer, stranger, employer, freelancer).is_err());
  }

  #[test]
  fn require_role_freelancer_only_allows_freelancer() {
    let (employer, freelancer, stranger) = ids();
    assert!(require_role(WorkflowRole::Freelancer, freelancer, employer, freelancer).is_ok());
    assert!(require_role(WorkflowRole::Freelancer, employer, employer, freelancer).is_err());
    assert!(require_role(WorkflowRole::Freelancer, stranger, employer, freelancer).is_err());
  }

  #[test]
  fn require_any_party_allows_both_blocks_stranger() {
    let (employer, freelancer, stranger) = ids();
    assert!(require_any_party(employer, employer, freelancer).is_ok());
    assert!(require_any_party(freelancer, employer, freelancer).is_ok());
    assert!(require_any_party(stranger, employer, freelancer).is_err());
  }

  #[test]
  fn require_post_creator_checks_person_id() {
    assert!(require_post_creator(PersonId(7), PersonId(7)).is_ok());
    assert!(require_post_creator(PersonId(8), PersonId(7)).is_err());
  }
}
