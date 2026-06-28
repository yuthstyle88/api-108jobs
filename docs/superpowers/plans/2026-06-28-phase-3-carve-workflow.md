# Phase 3 — Carve 5: `workflow_handlers` Crate

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract 2 workflow handler files from `crates/api/api/src/local_user/` into a new `crates/workflow_handlers/` crate named `app_108jobs_workflow_handlers`. Result: the HTTP adapter layer for workflow/escrow operations lives separately from the existing pure-domain `app_108jobs_workflow` crate.

**Architecture:** Physical file move of `workflow.rs` (10 handlers) and `workflow_authz.rs` (pure authz policy + 5 unit tests). The new crate depends on the existing `app_108jobs_workflow` crate (domain/service layer) — it is a thin HTTP adapter on top of it. The existing `crates/workflow/` (`app_108jobs_workflow`) is NOT modified.

**Tech Stack:** Rust, Cargo workspace, `cargo check`, `cargo nextest`

## Global Constraints

- **No behavior change.** Both moved files are byte-for-byte identical to the originals.
- **`app_108jobs_workflow` (existing domain crate) is NOT modified** — it must remain HTTP-agnostic.
- **Gate:** `cargo check --workspace` exits 0. `cargo nextest run -p app_108jobs_contract_tests` 16/16 pass.
- **Nightly fmt** after all changes.
- Commit: `refactor(phase-3): extract workflow HTTP handlers → app_108jobs_workflow_handlers`.

## Context

**Files to MOVE (2)** from `crates/api/api/src/local_user/` to `crates/workflow_handlers/src/`:

| File | Exports | Notes |
|------|---------|-------|
| `workflow.rs` | 10 handlers: `create_quotation`, `approve_quotation`, `submit_start_work`, `submit_work`, `approve_work`, `request_revision`, `update_budget_plan_status`, `start_workflow`, `cancel_job`, `get_billing_by_room` | Uses `super::workflow_authz` — must change to `crate::workflow_authz` |
| `workflow_authz.rs` | `WorkflowRole`, `caller_role`, `require_role`, `require_any_party`, `require_post_creator` + 5 unit tests | Pure functions, no HTTP deps |

**`super::workflow_authz` → `crate::workflow_authz`:** `workflow.rs` currently references the authz module as `super::workflow_authz` (because both live in `local_user/`). After the move both files are in `workflow_handlers/src/`, so the reference becomes `crate::workflow_authz`. This is the only internal reference that needs updating.

**Routes affected** (all in `/account/services/`):
```
POST /account/services/create-invoice
POST /account/services/approve-quotation
POST /account/services/start-workflow
POST /account/services/start-work
POST /account/services/submit-work
POST /account/services/request-revision
POST /account/services/approve-work
PUT  /account/services/budget-plan
GET  /account/services/billing/by-room
POST /account/services/cancel-job
```

**Dependencies for `app_108jobs_workflow_handlers` Cargo.toml:**
- `actix-web` (workspace)
- `app_108jobs_api_utils` (workspace) — `FastJobContext`, `LocalUserView`
- `app_108jobs_core` (workspace) — error types
- `app_108jobs_db` (workspace, features = ["full"]) — source models
- `app_108jobs_db_views_billing` (workspace) — DTOs
- `app_108jobs_db_views_local_user` (workspace, features = ["full"]) — `LocalUserView`
- `app_108jobs_workflow` (workspace) — `WorkflowService`, `WorkFlowOperationResponse`
- `chrono` (workspace)
- `serde_json` (workspace)

---

### Task 1: Create `crates/workflow_handlers/` and move files

**Files:**
- Create: `crates/workflow_handlers/Cargo.toml`
- Create: `crates/workflow_handlers/src/lib.rs`
- Copy: `workflow.rs` + `workflow_authz.rs` to `crates/workflow_handlers/src/`
- Edit: `crates/workflow_handlers/src/workflow.rs` — change `super::workflow_authz` → `crate::workflow_authz`
- Delete: source files from `crates/api/api/src/local_user/`
- Edit: `crates/api/api/src/local_user/mod.rs` — remove 2 `pub mod` lines
- Edit: root `Cargo.toml` — add member + workspace dep

**Interfaces:**
- Produces: `cargo check -p app_108jobs_workflow_handlers` → 0 errors

- [ ] **Step 1: Create directory**

```bash
mkdir -p crates/workflow_handlers/src
```

- [ ] **Step 2: Create `crates/workflow_handlers/Cargo.toml`**

```toml
[package]
name = "app_108jobs_workflow_handlers"
version = "1.0.0-alpha.5"
edition = "2021"
publish = false

[lib]
name = "app_108jobs_workflow_handlers"

[lints]
workspace = true

[dependencies]
app_108jobs_api_utils = { workspace = true }
app_108jobs_core = { workspace = true }
app_108jobs_db = { workspace = true, features = ["full"] }
app_108jobs_db_views_billing = { workspace = true }
app_108jobs_db_views_local_user = { workspace = true, features = ["full"] }
app_108jobs_workflow = { workspace = true }
actix-web = { workspace = true }
chrono = { workspace = true }
serde_json = { workspace = true }
```

Verify all dep names exist in root `Cargo.toml [workspace.dependencies]`. Also check whether `app_108jobs_db_views_billing` needs `features = ["full"]`:
```bash
grep "db_views_billing" crates/api/api/src/local_user/workflow.rs | head -3
grep "db_views_billing" Cargo.toml
```
If billing is used with ORM/Diesel types, add `features = ["full"]`.

- [ ] **Step 3: Create `crates/workflow_handlers/src/lib.rs`**

```rust
pub mod workflow;
pub mod workflow_authz;
```

- [ ] **Step 4: Copy handler files**

```bash
cp crates/api/api/src/local_user/workflow.rs      crates/workflow_handlers/src/workflow.rs
cp crates/api/api/src/local_user/workflow_authz.rs crates/workflow_handlers/src/workflow_authz.rs
```

- [ ] **Step 5: Fix the `super::` reference in workflow.rs**

Read `crates/workflow_handlers/src/workflow.rs` and find the `super::workflow_authz` reference:
```bash
grep -n "super::workflow_authz" crates/workflow_handlers/src/workflow.rs
```

Replace `super::workflow_authz` with `crate::workflow_authz` (should be 1-2 occurrences):
```bash
# Verify change
grep -n "workflow_authz" crates/workflow_handlers/src/workflow.rs
```

Expected: all references now use `crate::workflow_authz`.

- [ ] **Step 6: Remove 2 modules from `crates/api/api/src/local_user/mod.rs`**

Read the file, then remove:
```rust
pub mod workflow;
pub mod workflow_authz;
```

Delete source files:
```bash
rm crates/api/api/src/local_user/workflow.rs
rm crates/api/api/src/local_user/workflow_authz.rs
```

- [ ] **Step 7: Add to workspace root `Cargo.toml`**

In `[workspace]` members:
```toml
    "crates/workflow_handlers",
```

In `[workspace.dependencies]`:
```toml
app_108jobs_workflow_handlers = { version = "=1.0.0-alpha.5", path = "./crates/workflow_handlers" }
```

- [ ] **Step 8: Compile check**

```bash
cargo check -p app_108jobs_workflow_handlers 2>&1 | grep "^error" | head -10
```

Expected: 0 errors. Fix any missing deps or wrong feature flags using compiler output.

- [ ] **Step 9: Commit**

```bash
git add crates/workflow_handlers/ crates/api/api/src/local_user/ Cargo.toml Cargo.lock
git commit -m "refactor(phase-3): create crates/workflow_handlers/ with workflow HTTP handlers"
```

---

### Task 2: Wire routes + final gates

**Files:**
- Edit: root `Cargo.toml` `[dependencies]` — add `app_108jobs_workflow_handlers`
- Edit: `src/api_routes.rs` — update imports for 10 moved functions

**Interfaces:**
- Produces: `cargo check --workspace` → 0 errors; 16/16 contract tests pass

- [ ] **Step 1: Find current workflow imports in `api_routes.rs`**

```bash
grep -n "workflow" src/api_routes.rs
```

Read the relevant import block and all 10 route registrations to see exact import paths before changing.

- [ ] **Step 2: Add `app_108jobs_workflow_handlers` to root binary deps**

In root `Cargo.toml` `[dependencies]` (where `app_108jobs_identity` and `app_108jobs_payments` are listed):
```toml
app_108jobs_workflow_handlers = { workspace = true }
```

- [ ] **Step 3: Update `src/api_routes.rs`**

Add a new import block:
```rust
use app_108jobs_workflow_handlers::workflow::{
  approve_quotation,
  approve_work,
  cancel_job,
  create_quotation,
  get_billing_by_room,
  request_revision,
  start_workflow,
  submit_start_work,
  submit_work,
  update_budget_plan_status,
};
```

Remove those 10 functions from the `app_108jobs_api::local_user::workflow::{ ... }` import block.

Verify exact function names:
```bash
grep "^pub async fn" crates/workflow_handlers/src/workflow.rs
```

- [ ] **Step 4: Compile check**

```bash
cargo check --workspace 2>&1 | grep "^error" | head -10
```

Expected: 0 errors.

- [ ] **Step 5: Contract tests**

```bash
cargo nextest run -p app_108jobs_contract_tests 2>&1 | tail -3
```

Expected: 16/16 pass.

- [ ] **Step 6: Nightly fmt + commit**

```bash
cargo +nightly fmt
git add -A
git commit -m "refactor(phase-3): wire app_108jobs_workflow_handlers into api_routes"
```

---

## Self-Review

**Spec coverage:**
- ✅ `crates/workflow_handlers/` created as `app_108jobs_workflow_handlers`
- ✅ 2 files moved from `api::local_user` to `workflow_handlers`
- ✅ 2 files deleted from `api/local_user/`
- ✅ `super::workflow_authz` → `crate::workflow_authz` fixed
- ✅ `local_user/mod.rs` updated (2 `pub mod` lines removed)
- ✅ `src/api_routes.rs` imports updated to `app_108jobs_workflow_handlers::`
- ✅ `app_108jobs_workflow_handlers` in workspace + main binary deps
- ✅ Gate: `cargo check --workspace` + 16/16 contract tests green
- ✅ Existing `app_108jobs_workflow` (domain crate) NOT modified
- ✅ No behavior change — byte-for-byte identical handlers

**No placeholders.** Compile-driven: any missed reference is a compiler error.
