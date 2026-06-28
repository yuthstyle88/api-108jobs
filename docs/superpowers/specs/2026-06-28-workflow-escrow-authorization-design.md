# Workflow Escrow Authorization — Design

**Date:** 2026-06-28
**Branch:** `fix/workflow-escrow-authorization` (off `main`)
**Status:** Approved decisions — ready to implement
**Relates to:** Risk #1 in `2026-06-28-api-architecture-review.md` (§9). Independent of the rebuild.

---

## Problem

The escrow workflow transition handlers in
`crates/api/api/src/local_user/workflow.rs` do not authorize the caller:

- **No authenticated user at all** (5): `submit_start_work` (start-work),
  `submit_work`, `request_revision`, `approve_work`, `cancel_job`.
- **Has the user but no ownership/role check** (2): `create_quotation`
  (uses caller as issuer blindly), `approve_quotation` (uses caller as
  `employer_id` blindly).

Impact: any authenticated user who knows (or guesses) a `workflow_id` can drive
money-moving transitions — **`approve_work` releases escrow to the freelancer**
and **`cancel_job` refunds escrow** — for jobs they are not party to. This is a
financial-integrity hole (💰 High).

The correct pattern already exists in the same file: `get_billing_by_room`
(resolve `Billing`, allow only if `caller == freelancer_id || caller ==
employer_id`).

---

## Authorization matrix (locked with user 2026-06-28)

| Endpoint | Handler | Authorized caller |
|---|---|---|
| `POST /account/services/create-invoice` | `create_quotation` | **Freelancer** (party who is not the employer) |
| `POST /account/services/approve-quotation` | `approve_quotation` | **Employer** |
| `POST /account/services/start-work` | `submit_start_work` | **Freelancer** |
| `POST /account/services/submit-work` | `submit_work` | **Freelancer** |
| `POST /account/services/request-revision` | `request_revision` | **Employer** |
| `POST /account/services/approve-work` | `approve_work` | **Employer** |
| `POST /account/services/cancel-job` | `cancel_job` | **Either party** (employer or freelancer), any non-final state |
| `POST /account/services/start-workflow` | `start_workflow` | **Employer** (post creator) — secondary hardening |
| `PUT /account/services/budget-plan` | `update_budget_plan_status` | **Employer** (post creator) — secondary hardening |

- **No admin override.** A site admin has no special right to drive a job's
  transitions; dispute tooling, if ever needed, will be its own explicit,
  audited endpoint.

## Authorization basis

The two parties of a workflow are resolved as:

- **Post-billing (the common case):** `workflow.billing_id → Billing` gives
  `employer_id` and `freelancer_id` (both `LocalUserId`). Compare against
  `local_user_view.local_user.id`. This covers every transition except a
  cancel issued before a quotation exists.
- **Pre-billing (only `start_workflow`, `update_budget_plan_status`, and an
  early `cancel_job`):** `employer = post.creator` (via `workflow.post_id`,
  compared on `PersonId`); the freelancer is the other participant of
  `workflow.room_id` (via `ChatParticipant::list_participants_for_rooms`).

## Error semantics

- Caller is **not a party** → return **`FastJobErrorType::NotFound`** (do not
  leak that the workflow exists to non-parties), per the wrong-owner rule.
- Unauthenticated → handled by the `LocalUserView` extractor (401) once added.
- **Note:** the existing `get_billing_by_room` returns `NotAllowed` for the same
  situation. Aligning it to `NotFound` is **out of scope** here; flagged for a
  follow-up.

---

## Design — policy separated from IO

Add a small **pure policy module** so the authorization matrix is unit-testable
without a database (the api crate has no handler-test harness today):

```rust
// crates/api/api/src/local_user/workflow_authz.rs  (new)
pub enum WorkflowRole { Employer, Freelancer }

/// Caller's role for a workflow, given its billing row. None = not a party.
pub fn caller_role(caller: LocalUserId, billing: &Billing) -> Option<WorkflowRole>;

/// Ok(()) iff caller holds `required`; else Err(NotFound).
pub fn require_role(required: WorkflowRole, caller: LocalUserId, billing: &Billing)
  -> FastJobResult<()>;

/// Ok(()) iff caller is employer OR freelancer; else Err(NotFound).
pub fn require_any_party(caller: LocalUserId, billing: &Billing) -> FastJobResult<()>;
```

Handlers do the IO (load `Workflow` → `Billing`) then call the pure policy:

```rust
pub async fn approve_work(
  data: Json<ApproveWorkRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,          // <-- ADDED
) -> FastJobResult<Json<WorkFlowOperationResponse>> {
  let validated: ValidApproveWorkRequest = data.into_inner().try_into()?;
  let form = validated.0;
  let billing = Billing::read(&mut context.pool(), form.billing_id).await?;
  require_role(WorkflowRole::Employer, local_user_view.local_user.id, &billing)?;
  // …unchanged escrow-release logic…
}
```

For `cancel_job` (either party, possibly pre-billing): load `Workflow`; if
`billing_id` is `Some`, `require_any_party`; else verify caller is a participant
of `workflow.room_id` (or `caller.person.id == post.creator_id`), else
`NotFound`.

---

## Files touched

- **Create:** `crates/api/api/src/local_user/workflow_authz.rs` — pure policy +
  `#[cfg(test)]` unit tests.
- **Modify:** `crates/api/api/src/local_user/workflow.rs` — add `LocalUserView`
  to the 5 handlers missing it; add role/party checks to all 7 (+2 secondary);
  declare the new module.
- **Modify (1 line):** the module's parent `mod.rs` to add `mod workflow_authz;`
  if needed.
- **No migration. No wire-shape change** (requests/responses unchanged; only
  authorization is added → previously-allowed *wrong-party* calls now fail
  `NotFound`, previously-unauthenticated calls now require a token).

## Test plan (TDD)

Unit tests in `workflow_authz.rs` covering the matrix with synthetic `Billing`
rows (employer=1, freelancer=2, stranger=3):

1. `caller_role(1, billing) == Employer`; `caller_role(2, billing) == Freelancer`;
   `caller_role(3, billing) == None`.
2. `require_role(Employer, 1, b)` Ok; `require_role(Employer, 2, b)` Err(NotFound);
   `require_role(Employer, 3, b)` Err(NotFound).
3. `require_role(Freelancer, 2, b)` Ok; `require_role(Freelancer, 1, b)` Err.
4. `require_any_party(1|2, b)` Ok; `require_any_party(3, b)` Err(NotFound).

Build a `Billing` fixture via its struct literal (no DB). Run:
`cargo test -p app_108jobs_api`, then `cargo fmt` + `cargo clippy -- -D warnings`
on the crate.

## Out of scope (tracked in the review Risk List)

- Bugs #2–#8 (cancel+refund atomicity, platform-wallet identity, `user_review`
  participant check, SCB `qr_id` ownership, currency truncation, `TxKind`).
- Admin dispute tooling.
- Aligning `get_billing_by_room`'s `NotAllowed` → `NotFound`.

## Rollback

Pure additive code change, no schema/data change. Revert the commit to restore
prior behavior.
