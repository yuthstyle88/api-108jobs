# Phase 3 — Carve 6: `jobs` Crate

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract job/proposal interaction handlers from `crates/api/api/src/` into a new `crates/jobs/` crate named `app_108jobs_jobs`. This covers the `post/` directory (job interaction: like, save, list, feature, hide, lock, mark-read, link-metadata) and `comment/` directory (proposal interaction: like, save, distinguish, list), plus 4 `local_user/` list handlers (list_created, list_saved, list_liked, list_hidden).

**Architecture:** Move two entire subdirectories (`post/`, `comment/`) plus 4 loose files from `local_user/`. CRUD operations (create, get, update, delete, remove, report) live in `app_108jobs_api_crud` and are NOT touched. The new crate is `app_108jobs_jobs` — it exposes `pub mod post`, `pub mod comment`, and the 4 list modules.

**Tech Stack:** Rust, Cargo workspace, `cargo check`, `cargo nextest`

## Global Constraints

- **No behavior change.** All moved handlers byte-for-byte identical to originals.
- **CRUD handlers stay in `app_108jobs_api_crud`** — do not touch `crates/api/api_crud/`.
- **Gate:** `cargo check --workspace` exits 0. `cargo nextest run -p app_108jobs_contract_tests` 16/16 pass.
- **Nightly fmt** after all changes.

## Context

**Directories to MOVE wholesale:**
- `crates/api/api/src/post/` (11 files: mod.rs + 10 handler files) → `crates/jobs/src/post/`
- `crates/api/api/src/comment/` (6 files: mod.rs + 5 handler files) → `crates/jobs/src/comment/`

**Files to MOVE from `crates/api/api/src/local_user/`:**
- `list_created.rs` → `crates/jobs/src/list_created.rs`
- `list_saved.rs` → `crates/jobs/src/list_saved.rs`
- `list_liked.rs` → `crates/jobs/src/list_liked.rs`
- `list_hidden.rs` → `crates/jobs/src/list_hidden.rs`

**Handlers in `post/` (stay, all 11 files):**
`like_post`, `save_post`, `feature_post`, `hide_post`, `lock_post`, `mark_post_as_read`, `mark_posts_as_read`, `list_posts`, `update_post_notifications`, `get_link_metadata`, `list_post_likes`

**Handlers in `comment/` (stay, all 6 files):**
`like_comment`, `save_comment`, `distinguish_comment`, `list_comments`, `list_comments_slim`, `list_comment_likes`

**From `local_user/` (4 files):**
`list_person_created`, `list_person_saved`, `list_person_liked`, `list_person_hidden`

**Lines removed from `crates/api/api/src/lib.rs`:**
```rust
pub mod comment;
pub mod post;
```

**Lines removed from `crates/api/api/src/local_user/mod.rs`:**
```rust
pub mod list_created;
pub mod list_hidden;
pub mod list_liked;
pub mod list_saved;
```

**Expected dependencies for `app_108jobs_jobs` Cargo.toml** (verify with compiler — use feature = ["full"] on all db_views):
- `actix-web` (workspace)
- `app_108jobs_api_utils` (workspace)
- `app_108jobs_core` (workspace)
- `app_108jobs_db` (workspace, features = ["full"])
- `app_108jobs_db_views_comment` (workspace, features = ["full"])
- `app_108jobs_db_views_local_user` (workspace, features = ["full"])
- `app_108jobs_db_views_person_liked_combined` (workspace, features = ["full"])
- `app_108jobs_db_views_person_saved_combined` (workspace, features = ["full"])
- `app_108jobs_db_views_post` (workspace, features = ["full"])
- `app_108jobs_db_views_site` (workspace, features = ["full"])
- `app_108jobs_db_views_vote` (workspace, features = ["full"])
- `url` (workspace)
- `tracing` (workspace)
- `serde` (workspace)
- `chrono` (workspace)

---

### Task 1: Create `crates/jobs/` and move files

**Files:**
- Create: `crates/jobs/Cargo.toml`
- Create: `crates/jobs/src/lib.rs`
- Move: `crates/api/api/src/post/` → `crates/jobs/src/post/` (entire directory)
- Move: `crates/api/api/src/comment/` → `crates/jobs/src/comment/` (entire directory)
- Move: 4 files from `local_user/` to `crates/jobs/src/`
- Edit: `crates/api/api/src/lib.rs` — remove `pub mod comment; pub mod post;`
- Edit: `crates/api/api/src/local_user/mod.rs` — remove 4 `pub mod` lines
- Edit: root `Cargo.toml` — add member + workspace dep

**Interfaces:**
- Produces: `cargo check -p app_108jobs_jobs` → 0 errors

- [ ] **Step 1: Create directory structure**

```bash
mkdir -p crates/jobs/src/post crates/jobs/src/comment
```

- [ ] **Step 2: Create `crates/jobs/Cargo.toml`**

```toml
[package]
name = "app_108jobs_jobs"
version = "1.0.0-alpha.5"
edition = "2021"
publish = false

[lib]
name = "app_108jobs_jobs"

[lints]
workspace = true

[dependencies]
app_108jobs_api_utils = { workspace = true }
app_108jobs_core = { workspace = true }
app_108jobs_db = { workspace = true, features = ["full"] }
app_108jobs_db_views_comment = { workspace = true, features = ["full"] }
app_108jobs_db_views_local_user = { workspace = true, features = ["full"] }
app_108jobs_db_views_person_liked_combined = { workspace = true, features = ["full"] }
app_108jobs_db_views_person_saved_combined = { workspace = true, features = ["full"] }
app_108jobs_db_views_post = { workspace = true, features = ["full"] }
app_108jobs_db_views_site = { workspace = true, features = ["full"] }
app_108jobs_db_views_vote = { workspace = true, features = ["full"] }
actix-web = { workspace = true }
chrono = { workspace = true }
serde = { workspace = true }
tracing = { workspace = true }
url = { workspace = true }
```

- [ ] **Step 3: Create `crates/jobs/src/lib.rs`**

```rust
pub mod comment;
pub mod list_created;
pub mod list_hidden;
pub mod list_liked;
pub mod list_saved;
pub mod post;
```

- [ ] **Step 4: Copy `post/` directory**

```bash
cp -r crates/api/api/src/post/. crates/jobs/src/post/
```

Verify:
```bash
ls crates/jobs/src/post/
```
Expected: `mod.rs` + 10 handler `.rs` files.

- [ ] **Step 5: Copy `comment/` directory**

```bash
cp -r crates/api/api/src/comment/. crates/jobs/src/comment/
```

Verify:
```bash
ls crates/jobs/src/comment/
```
Expected: `mod.rs` + 5 handler `.rs` files.

- [ ] **Step 6: Copy 4 local_user list files**

```bash
cp crates/api/api/src/local_user/list_created.rs crates/jobs/src/list_created.rs
cp crates/api/api/src/local_user/list_saved.rs   crates/jobs/src/list_saved.rs
cp crates/api/api/src/local_user/list_liked.rs   crates/jobs/src/list_liked.rs
cp crates/api/api/src/local_user/list_hidden.rs  crates/jobs/src/list_hidden.rs
```

- [ ] **Step 7: Check for `crate::` internal references**

```bash
grep -rn "^use crate::\|crate::" crates/jobs/src/ | grep -v "//.*crate::"
```

If any `crate::` references appear, identify which helper they reference. If the helper lives in `crates/api/api/src/lib.rs` (e.g., `check_report_reason`), check whether these handlers actually use it or not. The likely candidates are:
- `check_report_reason` — only used by report handlers (in api_crud), should NOT appear here
- Any `crate::post::` or `crate::comment::` cross-references within the same moved module — these become `super::` or need updating

Fix any `crate::` references that break the crate boundary.

- [ ] **Step 8: Remove moved modules from `crates/api/api/src/lib.rs`**

Read the file. Remove these two lines:
```rust
pub mod comment;
pub mod post;
```

Delete source files:
```bash
rm -r crates/api/api/src/post/
rm -r crates/api/api/src/comment/
```

- [ ] **Step 9: Remove 4 modules from `crates/api/api/src/local_user/mod.rs`**

Read the file. Remove:
```rust
pub mod list_created;
pub mod list_hidden;
pub mod list_liked;
pub mod list_saved;
```

Delete source files:
```bash
rm crates/api/api/src/local_user/list_created.rs
rm crates/api/api/src/local_user/list_hidden.rs
rm crates/api/api/src/local_user/list_liked.rs
rm crates/api/api/src/local_user/list_saved.rs
```

- [ ] **Step 10: Add to workspace root `Cargo.toml`**

Members:
```toml
    "crates/jobs",
```

Workspace.dependencies:
```toml
app_108jobs_jobs = { version = "=1.0.0-alpha.5", path = "./crates/jobs" }
```

- [ ] **Step 11: Compile check**

```bash
cargo check -p app_108jobs_jobs 2>&1 | grep "^error" | head -20
```

Expected: 0 errors. Fix any missing deps or wrong feature flags using compiler output. The compiler will tell you exactly which dep is missing.

- [ ] **Step 12: Commit**

```bash
git add crates/jobs/ crates/api/api/src/lib.rs crates/api/api/src/local_user/ Cargo.toml Cargo.lock
git commit -m "refactor(phase-3): create crates/jobs/ with post+comment+list handler modules"
```

---

### Task 2: Wire routes + final gates

**Files:**
- Edit: root `Cargo.toml` `[dependencies]` — add `app_108jobs_jobs`
- Edit: `src/api_routes.rs` — update 15 imports across 3 import blocks

**Interfaces:**
- Produces: `cargo check --workspace` → 0 errors; 16/16 contract tests pass

- [ ] **Step 1: Read current imports in `api_routes.rs`**

```bash
grep -n "post::\|comment::\|list_created\|list_saved\|list_liked\|list_hidden" src/api_routes.rs | head -30
```

Note the exact current import blocks for these handlers. You need to remove them from `app_108jobs_api::{ post::..., comment::..., local_user::{ list_created::..., ... } }` and add them from `app_108jobs_jobs`.

- [ ] **Step 2: Add `app_108jobs_jobs` to root binary deps**

In root `Cargo.toml` `[dependencies]` (alongside `app_108jobs_identity`, `app_108jobs_payments`, `app_108jobs_workflow_handlers`):
```toml
app_108jobs_jobs = { workspace = true }
```

- [ ] **Step 3: Update `src/api_routes.rs`**

Add a new import block (before or after the existing domain crate imports):
```rust
use app_108jobs_jobs::{
  comment::{
    distinguish::distinguish_comment,
    like::like_comment,
    list::{list_comments, list_comments_slim},
    list_comment_likes::list_comment_likes,
    save::save_comment,
  },
  list_created::list_person_created,
  list_hidden::list_person_hidden,
  list_liked::list_person_liked,
  list_saved::list_person_saved,
  post::{
    feature::feature_post,
    get_link_metadata::get_link_metadata,
    hide::hide_post,
    like::like_post,
    list::list_posts,
    list_post_likes::list_post_likes,
    lock::lock_post,
    mark_many_read::mark_posts_as_read,
    mark_read::mark_post_as_read,
    save::save_post,
    update_notifications::update_post_notifications,
  },
};
```

Verify exact function names first:
```bash
grep "^pub async fn\|^pub fn" crates/jobs/src/post/*.rs crates/jobs/src/comment/*.rs crates/jobs/src/list_*.rs
```

Remove the same functions from the `app_108jobs_api::{ post::..., comment::..., local_user::{ ... } }` import blocks.

- [ ] **Step 4: Compile check**

```bash
cargo check --workspace 2>&1 | grep "^error" | head -20
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
git commit -m "refactor(phase-3): wire app_108jobs_jobs into api_routes"
```

---

## Self-Review

**Spec coverage:**
- ✅ `crates/jobs/` created as `app_108jobs_jobs`
- ✅ `post/` directory moved (11 files)
- ✅ `comment/` directory moved (6 files)
- ✅ 4 `list_*.rs` files moved from `local_user/`
- ✅ `pub mod post; pub mod comment;` removed from `api/src/lib.rs`
- ✅ 4 `pub mod list_*` lines removed from `local_user/mod.rs`
- ✅ Source directories/files deleted from api crate
- ✅ `src/api_routes.rs` imports updated to `app_108jobs_jobs::`
- ✅ `app_108jobs_jobs` in workspace + main binary deps
- ✅ Gate: `cargo check --workspace` + 16/16 contract tests green
- ✅ CRUD handlers in `api_crud` untouched
- ✅ No behavior change — byte-for-byte identical handlers

**No placeholders.** Compile-driven: any missed reference is a compiler error.
