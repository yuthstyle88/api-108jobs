# Phase 3 — Carve 3: `identity` Crate

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract the 14 authentication/identity handler files from `crates/api/api/src/local_user/` into a new `crates/identity/` crate named `app_108jobs_identity`. The result: a clear domain boundary where all core auth flows (login, JWT refresh, TOTP, email verification, password management) live in one crate.

**Architecture:** Physical file move of 14 handler files from `app_108jobs_api::local_user::*` to `app_108jobs_identity::*`. The identity crate depends on `app_108jobs_api_utils` (for `FastJobContext` and `Claims`), keeping JWT logic in place. The 32 non-identity local_user handlers (wallet, bank_account, workflow, notifications, etc.) stay in `app_108jobs_api` for future domain carves. Route registration in `src/api_routes.rs` is updated to import identity handlers from `app_108jobs_identity`.

**Tech Stack:** Rust, Cargo workspace, `cargo check`, `cargo nextest`

## Global Constraints

- **No behavior change.** The 14 moved handlers are byte-for-byte identical to the originals; only their crate changes.
- **No module path changes visible externally.** `src/api_routes.rs` is the only external caller — it is updated to import from `app_108jobs_identity`.
- **`claims.rs` stays in `app_108jobs_api_utils`.** The `identity` crate imports `Claims` from `api_utils` (no circular dep: `identity` → `api_utils`, never reversed).
- **Gate:** `cargo check --workspace` exits 0. `cargo nextest run -p app_108jobs_contract_tests` 16/16 pass.
- **Nightly fmt** after all changes.
- Commit: `refactor(phase-3): extract auth handlers from api::local_user → app_108jobs_identity`.

## Context

**Files to MOVE (14)** from `crates/api/api/src/local_user/` to `crates/identity/src/`:

| File | Domain |
|------|--------|
| `login.rs` | Core auth |
| `logout.rs` | Core auth |
| `refresh.rs` | JWT refresh |
| `validate_auth.rs` | Auth middleware helper |
| `list_logins.rs` | Session listing |
| `verify_email.rs` | Email verification |
| `resend_verification_email.rs` | Email verification |
| `generate_totp_secret.rs` | TOTP setup |
| `update_totp.rs` | TOTP enable/verify |
| `get_captcha.rs` | Captcha challenge |
| `change_password.rs` | Password management |
| `reset_password.rs` | Password reset request |
| `change_password_after_reset.rs` | Password reset completion |
| `identity_card.rs` | KYC document upload |

**Files to KEEP** in `crates/api/api/src/local_user/` (all 32 other handlers):
`add_admin`, `ban_person`, `bank_account`, `block`, `contact`, `donation_dialog_shown`, `exchange`, `export_data`, `list_created`, `list_hidden`, `list_liked`, `list_media`, `list_read`, `list_saved`, `list_top_up_requests`, `note_person`, `notifications/*`, `profile`, `report_count`, `review`, `save_settings`, `update_term`, `user_block_instance`, `wallet`, `withdraw`, `workflow`, `workflow_authz`

**Dependencies for `app_108jobs_identity` Cargo.toml** (gathered from file imports):
- `actix-web`
- `app_108jobs_api_utils` (for `FastJobContext`, `Claims`)
- `app_108jobs_core` (for `FastJobResult`, `FastJobErrorType`, settings)
- `app_108jobs_db` (features = ["full"]) — for `LoginToken`, `LocalUser`, etc.
- `app_108jobs_db_views_local_user` (features = ["full"])
- `app_108jobs_db_views_site` (features = ["full"])
- `app_108jobs_email`
- `bcrypt`
- `captcha`
- `chrono`
- `serde`
- `tracing`

---

### Task 1: Create `crates/identity/` and move files

**Files:**
- Create: `crates/identity/Cargo.toml`
- Create: `crates/identity/src/lib.rs`
- Move (copy then delete): 14 handler `.rs` files to `crates/identity/src/`
- Modify: `crates/api/api/src/local_user/mod.rs` — remove 14 `pub mod` declarations
- Modify: root `Cargo.toml` — add `crates/identity` to workspace members + add `app_108jobs_identity` to workspace.dependencies

**Interfaces:**
- Produces: `app_108jobs_identity` crate compiles (handlers are identical to originals; dependencies satisfied)

- [ ] **Step 1: Create `crates/identity/` directory structure**

```bash
mkdir -p crates/identity/src
```

- [ ] **Step 2: Create `crates/identity/Cargo.toml`**

```toml
[package]
name = "app_108jobs_identity"
version = "1.0.0-alpha.5"
edition = "2021"

[lib]
name = "app_108jobs_identity"

[lints]
workspace = true

[dependencies]
app_108jobs_api_utils = { workspace = true }
app_108jobs_core = { workspace = true }
app_108jobs_db = { workspace = true, features = ["full"] }
app_108jobs_db_views_local_user = { workspace = true, features = ["full"] }
app_108jobs_db_views_site = { workspace = true, features = ["full"] }
app_108jobs_email = { workspace = true }
actix-web = { workspace = true }
bcrypt = { workspace = true }
captcha = { workspace = true }
chrono = { workspace = true }
serde = { workspace = true }
tracing = { workspace = true }
```

Verify the exact versions match workspace deps by checking root `Cargo.toml`. Adjust any that are missing or have different names.

- [ ] **Step 3: Create `crates/identity/src/lib.rs`**

```rust
pub mod change_password;
pub mod change_password_after_reset;
pub mod generate_totp_secret;
pub mod get_captcha;
pub mod identity_card;
pub mod list_logins;
pub mod login;
pub mod logout;
pub mod refresh;
pub mod resend_verification_email;
pub mod reset_password;
pub mod update_totp;
pub mod validate_auth;
pub mod verify_email;
```

- [ ] **Step 4: Copy handler files to `crates/identity/src/`**

```bash
cd /path/to/api-108jobs  # use the actual working directory
for f in \
  change_password \
  change_password_after_reset \
  generate_totp_secret \
  get_captcha \
  identity_card \
  list_logins \
  login \
  logout \
  refresh \
  resend_verification_email \
  reset_password \
  update_totp \
  validate_auth \
  verify_email; do
  cp "crates/api/api/src/local_user/${f}.rs" "crates/identity/src/${f}.rs"
done
```

- [ ] **Step 5: Verify no internal imports need fixing**

The files in `crates/identity/src/` reference external crates (`app_108jobs_api_utils`, `app_108jobs_db`, etc.) not internal (`crate::`) paths. Run:

```bash
grep -n "^use crate::" crates/identity/src/*.rs
```

Expected: zero matches. If any `use crate::` appears, identify the module and expose it from the identity crate's lib.rs.

- [ ] **Step 6: Remove the 14 modules from `crates/api/api/src/local_user/mod.rs`**

Read `crates/api/api/src/local_user/mod.rs` first. Remove these lines:
```rust
pub mod change_password;
pub mod change_password_after_reset;
pub mod generate_totp_secret;
pub mod get_captcha;
pub mod identity_card;
pub mod list_logins;
pub mod login;
pub mod logout;
pub mod refresh;
pub mod resend_verification_email;
pub mod reset_password;
pub mod update_totp;
pub mod validate_auth;
pub mod verify_email;
```

And delete the 14 source files from `crates/api/api/src/local_user/`:
```bash
for f in change_password change_password_after_reset generate_totp_secret get_captcha identity_card list_logins login logout refresh resend_verification_email reset_password update_totp validate_auth verify_email; do
  rm "crates/api/api/src/local_user/${f}.rs"
done
```

- [ ] **Step 7: Add `identity` to workspace root `Cargo.toml`**

In `[workspace]` members, add:
```toml
    "crates/identity",
```

In `[workspace.dependencies]`, add:
```toml
app_108jobs_identity = { version = "=1.0.0-alpha.5", path = "./crates/identity" }
```

- [ ] **Step 8: Check crate compiles**

```bash
cargo check -p app_108jobs_identity 2>&1 | grep "^error" | head -10
```

Fix any errors from missing deps or wrong feature flags. Expected: 0 errors.

Also check the api crate still compiles with the 14 modules removed:
```bash
cargo check -p app_108jobs_api 2>&1 | grep "^error" | head -10
```

Expected: 0 errors (handlers removed, routes not yet updated — this is fine at Task 1 stage, the main binary may have import errors which are fixed in Task 2).

- [ ] **Step 9: Commit the new crate**

```bash
git add crates/identity/ crates/api/api/src/local_user/mod.rs
git add Cargo.toml Cargo.lock
for f in change_password change_password_after_reset generate_totp_secret get_captcha identity_card list_logins login logout refresh resend_verification_email reset_password update_totp validate_auth verify_email; do
  git add "crates/api/api/src/local_user/${f}.rs"
done
git commit -m "refactor(phase-3): create crates/identity/ with 14 auth handlers from api::local_user"
```

---

### Task 2: Update route registration + final gates

**Files:**
- Modify: `src/api_routes.rs` — update the 14 identity handler imports
- Modify: root binary `Cargo.toml` (if the main binary is a separate crate) OR root `Cargo.toml` `[dependencies]` section — add `app_108jobs_identity` dep

**Interfaces:**
- Produces: `cargo check --workspace` exits 0; 16/16 contract tests pass

- [ ] **Step 1: Find the main binary's dependency list**

```bash
grep -n "app_108jobs_api\b" Cargo.toml | head -5
```

The main binary's dependencies are in the root `Cargo.toml` `[dependencies]` section (not workspace deps). Find the line that adds `app_108jobs_api` and add `app_108jobs_identity` next to it:
```toml
app_108jobs_api = { workspace = true }
app_108jobs_identity = { workspace = true }
```

- [ ] **Step 2: Update `src/api_routes.rs` imports**

Read `src/api_routes.rs`. Find the import block for the 14 identity handlers. Currently they are spread across the `app_108jobs_api::local_user::{ ... }` block.

Add a new import block for identity handlers BEFORE the existing `use app_108jobs_api::` block:
```rust
use app_108jobs_identity::{
  change_password::change_password,
  change_password_after_reset::change_password_after_reset,
  generate_totp_secret::generate_totp_secret,
  get_captcha::get_captcha,
  identity_card::{delete_identity_card, get_identity_card, submit_identity_card},
  list_logins::list_logins,
  login::login,
  logout::logout,
  refresh::refresh_token,
  resend_verification_email::resend_verification_email,
  reset_password::reset_password,
  update_totp::update_totp,
  validate_auth::validate_auth,
  verify_email::verify_email,
};
```

Note: Check the exact function names exported by each file — some files may export multiple functions (e.g., `identity_card.rs` may have multiple public functions). Run:
```bash
grep "^pub fn\|^pub async fn" crates/identity/src/identity_card.rs
grep "^pub fn\|^pub async fn" crates/identity/src/login.rs
```
Use the exact function names found.

Then remove those 14 items from the `app_108jobs_api::local_user::{ ... }` import block in `api_routes.rs`.

- [ ] **Step 3: Workspace compile check**

```bash
cargo check --workspace 2>&1 | grep "^error" | head -20
```

Expected: 0 errors. If there are import errors, they will be from function names that differ between the plan and the actual files — fix each one using the compiler output.

Verify zero residual references in api routes:
```bash
grep -n "local_user::{.*login\|local_user::{.*logout\|local_user::{.*captcha\|local_user::{.*totp\|local_user::{.*password\|local_user::{.*verify\|local_user::{.*refresh\|local_user::{.*validate" src/api_routes.rs
```
Expected: 0 matches (all 14 moved handlers no longer imported from `local_user`).

- [ ] **Step 4: Run contract tests**

```bash
cargo nextest run -p app_108jobs_contract_tests 2>&1 | tail -3
```

Expected: 16/16 pass.

- [ ] **Step 5: Nightly fmt + final commit**

```bash
cargo +nightly fmt
git add -A
git commit -m "refactor(phase-3): wire app_108jobs_identity handlers into api_routes"
```

---

## Self-Review

**Spec coverage:**
- ✅ `crates/identity/` created with `app_108jobs_identity` crate name
- ✅ 14 auth handler files moved from `api::local_user` to `identity`
- ✅ 14 files DELETED from `crates/api/api/src/local_user/`
- ✅ `local_user/mod.rs` updated (14 `pub mod` lines removed)
- ✅ `src/api_routes.rs` imports updated to `app_108jobs_identity::`
- ✅ `app_108jobs_identity` added to workspace members + workspace.dependencies
- ✅ `app_108jobs_identity` added to main binary deps
- ✅ Gate: `cargo check --workspace` + 16/16 contract tests green
- ✅ Non-identity local_user handlers remain in `app_108jobs_api::local_user`
- ✅ `claims.rs` stays in `app_108jobs_api_utils` (no circular dep)

**No placeholders.** Compile-driven: any missed reference surfaces as a compiler error.
