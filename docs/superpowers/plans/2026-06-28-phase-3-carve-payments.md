# Phase 3 — Carve 4: `payments` Crate

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract the 4 wallet/payment handler files from `crates/api/api/src/local_user/` into a new `crates/payments/` crate named `app_108jobs_payments`. Result: a clear domain boundary for all wallet, bank-account, withdrawal, and top-up operations.

**Architecture:** Physical file move of 4 handler files. The payments crate depends on `app_108jobs_api_utils` (for `FastJobContext` and helper functions), `app_108jobs_db_views_wallet`, and `app_108jobs_db_views_bank_account`. Admin payment handlers (`admin/wallet.rs`, `admin/bank_account.rs`, `admin/currency.rs`, `admin/platform.rs`) and the SCB integration (`crates/routes/src/payments/`) are out of scope for this carve — they move with their respective future carves (`admin` and `http`).

**Tech Stack:** Rust, Cargo workspace, `cargo check`, `cargo nextest`

## Global Constraints

- **No behavior change.** All 4 moved handlers are byte-for-byte identical.
- **Gate:** `cargo check --workspace` exits 0. `cargo nextest run -p app_108jobs_contract_tests` 16/16 pass.
- **Nightly fmt** after all changes.
- Commit: `refactor(phase-3): extract wallet/bank/withdraw handlers → app_108jobs_payments`.

## Context

**Files to MOVE (4)** from `crates/api/api/src/local_user/` to `crates/payments/src/`:

| File | Exports | Domain |
|------|---------|--------|
| `wallet.rs` | `get_wallet`, `deposit_wallet` | Wallet read/deposit |
| `bank_account.rs` | `create_bank_account`, `list_user_bank_accounts`, `set_default_bank_account`, `update_bank_account`, `delete_bank_account`, `list_banks` | Bank account CRUD |
| `withdraw.rs` | `submit_withdraw`, `list_withdraw_requests`, `retract_withdraw` | Withdrawal requests |
| `list_top_up_requests.rs` | `list_top_up_requests` | Top-up listing |

**Files to KEEP** in `crates/api/api/src/local_user/`: everything else (46 - 13 moved to identity - 4 moved here = 29 remaining).

**Dependencies for `app_108jobs_payments` Cargo.toml:**
- `actix-web` (workspace)
- `app_108jobs_api_common` (workspace) — `bank_account::BankAccountOperationResponse`
- `app_108jobs_api_utils` (workspace) — `FastJobContext`, `utils::ensure_bank_account_unique_for_user`, `utils::list_withdraw_requests_inner`, `utils::list_top_up_requests_inner`
- `app_108jobs_core` (workspace) — error types
- `app_108jobs_db` (workspace, features = ["full"]) — source models
- `app_108jobs_db_views_bank_account` (workspace, features = ["full"])
- `app_108jobs_db_views_local_user` (workspace, features = ["full"]) — `LocalUserView`
- `app_108jobs_db_views_site` (workspace, features = ["full"]) — `SuccessResponse`
- `app_108jobs_db_views_wallet` (workspace, features = ["full"])
- `uuid` (workspace)

---

### Task 1: Create `crates/payments/` and move files

**Files:**
- Create: `crates/payments/Cargo.toml`
- Create: `crates/payments/src/lib.rs`
- Move (copy+delete): 4 handler files to `crates/payments/src/`
- Modify: `crates/api/api/src/local_user/mod.rs` — remove 4 `pub mod` lines
- Modify: root `Cargo.toml` — add member + workspace dep

**Interfaces:**
- Produces: `cargo check -p app_108jobs_payments` → 0 errors

- [ ] **Step 1: Create directory**

```bash
mkdir -p crates/payments/src
```

- [ ] **Step 2: Create `crates/payments/Cargo.toml`**

```toml
[package]
name = "app_108jobs_payments"
version = "1.0.0-alpha.5"
edition = "2021"
publish = false

[lib]
name = "app_108jobs_payments"

[lints]
workspace = true

[dependencies]
app_108jobs_api_common = { workspace = true }
app_108jobs_api_utils = { workspace = true }
app_108jobs_core = { workspace = true }
app_108jobs_db = { workspace = true, features = ["full"] }
app_108jobs_db_views_bank_account = { workspace = true, features = ["full"] }
app_108jobs_db_views_local_user = { workspace = true, features = ["full"] }
app_108jobs_db_views_site = { workspace = true, features = ["full"] }
app_108jobs_db_views_wallet = { workspace = true, features = ["full"] }
actix-web = { workspace = true }
uuid = { workspace = true }
```

Verify all dep names match workspace by reading root `Cargo.toml [workspace.dependencies]`.

- [ ] **Step 3: Create `crates/payments/src/lib.rs`**

```rust
pub mod bank_account;
pub mod list_top_up_requests;
pub mod wallet;
pub mod withdraw;
```

- [ ] **Step 4: Copy 4 handler files**

```bash
cp crates/api/api/src/local_user/wallet.rs              crates/payments/src/wallet.rs
cp crates/api/api/src/local_user/bank_account.rs        crates/payments/src/bank_account.rs
cp crates/api/api/src/local_user/withdraw.rs            crates/payments/src/withdraw.rs
cp crates/api/api/src/local_user/list_top_up_requests.rs crates/payments/src/list_top_up_requests.rs
```

- [ ] **Step 5: Verify no `crate::` internal references**

```bash
grep "^use crate::" crates/payments/src/*.rs
```

Expected: 0 results. If any appear, trace the helper and resolve it within `payments`.

- [ ] **Step 6: Remove 4 modules from `crates/api/api/src/local_user/mod.rs`**

Remove these lines (read the file first to confirm exact content):
```rust
pub mod bank_account;
pub mod list_top_up_requests;
pub mod wallet;
pub mod withdraw;
```

Then delete the 4 source files:
```bash
rm crates/api/api/src/local_user/wallet.rs
rm crates/api/api/src/local_user/bank_account.rs
rm crates/api/api/src/local_user/withdraw.rs
rm crates/api/api/src/local_user/list_top_up_requests.rs
```

- [ ] **Step 7: Add to workspace root `Cargo.toml`**

Members:
```toml
    "crates/payments",
```

Workspace.dependencies:
```toml
app_108jobs_payments = { version = "=1.0.0-alpha.5", path = "./crates/payments" }
```

- [ ] **Step 8: Compile check**

```bash
cargo check -p app_108jobs_payments 2>&1 | grep "^error" | head -10
```

Expected: 0 errors. Fix any missing deps or wrong feature flags using compiler output.

- [ ] **Step 9: Commit**

```bash
git add crates/payments/ crates/api/api/src/local_user/ Cargo.toml Cargo.lock
git commit -m "refactor(phase-3): create crates/payments/ with wallet/bank_account/withdraw handlers"
```

---

### Task 2: Wire routes + final gates

**Files:**
- Modify: root `Cargo.toml` `[dependencies]` — add `app_108jobs_payments`
- Modify: `src/api_routes.rs` — update imports for 11 moved functions

**Interfaces:**
- Produces: `cargo check --workspace` → 0 errors; 16/16 contract tests pass

- [ ] **Step 1: Add `app_108jobs_payments` to root binary deps**

In root `Cargo.toml` `[dependencies]` section (where `app_108jobs_api` and `app_108jobs_identity` are listed), add:
```toml
app_108jobs_payments = { workspace = true }
```

- [ ] **Step 2: Update `src/api_routes.rs`**

Read the file. The 11 payment functions are currently imported from `app_108jobs_api::local_user::{ bank_account::*, wallet::*, withdraw::*, list_top_up_requests::* }`.

Add a new import block:
```rust
use app_108jobs_payments::{
  bank_account::{
    create_bank_account,
    delete_bank_account,
    list_banks,
    list_user_bank_accounts,
    set_default_bank_account,
    update_bank_account,
  },
  list_top_up_requests::list_top_up_requests,
  wallet::{deposit_wallet, get_wallet},
  withdraw::{list_withdraw_requests, retract_withdraw, submit_withdraw},
};
```

Remove the same 11 items from the `app_108jobs_api::local_user::{ ... }` block.

Verify exact function names by reading the source files:
```bash
grep "^pub async fn" crates/payments/src/wallet.rs crates/payments/src/withdraw.rs
```

- [ ] **Step 3: Compile check**

```bash
cargo check --workspace 2>&1 | grep "^error" | head -10
```

Expected: 0 errors.

- [ ] **Step 4: Contract tests**

```bash
cargo nextest run -p app_108jobs_contract_tests 2>&1 | tail -3
```

Expected: 16/16 pass.

- [ ] **Step 5: Nightly fmt + commit**

```bash
cargo +nightly fmt
git add -A
git commit -m "refactor(phase-3): wire app_108jobs_payments into api_routes"
```

---

## Self-Review

**Spec coverage:**
- ✅ `crates/payments/` created as `app_108jobs_payments`
- ✅ 4 files moved from `api::local_user` to `payments`
- ✅ 4 files deleted from `api/local_user/`
- ✅ `local_user/mod.rs` updated (4 `pub mod` lines removed)
- ✅ `src/api_routes.rs` imports updated to `app_108jobs_payments::`
- ✅ `app_108jobs_payments` in workspace + main binary deps
- ✅ Gate: `cargo check --workspace` + 16/16 contract tests green
- ✅ Admin payment handlers and SCB integration left in place (out of scope)
- ✅ No behavior change — byte-for-byte identical handlers

**No placeholders.** Compile-driven: any missed reference is a compiler error.
