# Phase 3 — Carve 6: `jobs` + `proposals` Crates

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract job-listing handlers (from `crates/api/api/src/post/` + `crates/api/api_crud/src/post/`) into `crates/jobs/` (`app_108jobs_jobs`) and proposal handlers (from `crates/api/api/src/comment/` + `crates/api/api_crud/src/comment/`) into `crates/proposals/` (`app_108jobs_proposals`). Both crates get the current Rust names — the comment→proposal DB rename is a separate phase.

**Architecture:** Two new crates, each combining their handler + CRUD source files. The existing `api` and `api_crud` crates have the modules in top-level `src/post/` and `src/comment/` directories (not inside `local_user/`). After moving, `api_crud` may become empty of real content (check at the end).

**Tech Stack:** Rust, Cargo workspace, `cargo check`, `cargo nextest`

## Global Constraints

- **No behavior change.** All moved handlers are byte-for-byte identical.
- **Gate:** `cargo check --workspace` exits 0. `cargo nextest run -p app_108jobs_contract_tests` 16/16 pass.
- **Nightly fmt** after all changes.
- **DB rename (comment→proposal) is NOT part of this carve** — that is a separate phase.

## Context

### Files to MOVE into `crates/jobs/src/`

From `crates/api/api/src/post/`:
- `feature.rs` — `feature_post()`
- `get_link_metadata.rs` — `get_link_metadata()`
- `hide.rs` — `hide_post()`
- `like.rs` — `like_post()`
- `list.rs` — `list_posts()`
- `list_post_likes.rs` — `list_post_likes()`
- `lock.rs` — `lock_post()`
- `mark_many_read.rs` — `mark_posts_as_read()`
- `mark_read.rs` — `mark_post_as_read()`
- `save.rs` — `save_post()`
- `update_notifications.rs` — `update_post_notifications()`

From `crates/api/api_crud/src/post/`:
- `create.rs` — `create_post()`
- `read.rs` — `get_post()`
- `update.rs` — `update_post()`
- `delete.rs` — `delete_post()`
- `remove.rs` — `remove_post()`
- `mod.rs` — `check_post_deleted_or_removed()` helper (keep as `mod.rs` or move to lib.rs)

### Files to MOVE into `crates/proposals/src/`

From `crates/api/api/src/comment/`:
- `distinguish.rs` — `distinguish_comment()`
- `like.rs` — `like_comment()`
- `list.rs` — `list_comments()`, `list_comments_slim()`
- `list_comment_likes.rs` — `list_comment_likes()`
- `save.rs` — `save_comment()`

From `crates/api/api_crud/src/comment/`:
- `create.rs` — `create_comment()`
- `read.rs` — `get_comment()`
- `update.rs` — `update_comment()`
- `delete.rs` — `delete_comment()`
- `remove.rs` — `remove_comment()`

### Internal References to Check

```bash
# Check for internal cross-references between post and comment modules
grep -rn "super::\|crate::" crates/api/api/src/post/ crates/api/api/src/comment/
grep -rn "super::\|crate::" crates/api/api_crud/src/post/ crates/api/api_crud/src/comment/
# Check for check_post_deleted_or_removed usage
grep -rn "check_post_deleted_or_removed" crates/api/api_crud/src/
```

### Dependencies for `app_108jobs_jobs` Cargo.toml

Read `crates/api/api/src/post/*.rs` and `crates/api/api_crud/src/post/*.rs` imports to confirm. Expected:
- `actix-web` (workspace)
- `app_108jobs_api_common` (workspace)
- `app_108jobs_api_utils` (workspace)
- `app_108jobs_core` (workspace)
- `app_108jobs_db` (workspace, features = ["full"])
- `app_108jobs_db_views_category` (workspace, features = ["full"])
- `app_108jobs_db_views_local_user` (workspace, features = ["full"])
- `app_108jobs_db_views_post` (workspace, features = ["full"])
- `app_108jobs_db_views_search_combined` (workspace, features = ["full"])
- `app_108jobs_db_views_site` (workspace, features = ["full"])
- `app_108jobs_db_views_vote` (workspace, features = ["full"])
- `anyhow` (workspace)
- `chrono` (workspace)
- `serde` (workspace)
- `serde_json` (workspace)
- `tracing` (workspace)
- `url` (workspace)
- `uuid` (workspace)

### Dependencies for `app_108jobs_proposals` Cargo.toml

- `actix-web` (workspace)
- `app_108jobs_api_common` (workspace)
- `app_108jobs_api_utils` (workspace)
- `app_108jobs_core` (workspace)
- `app_108jobs_db` (workspace, features = ["full"])
- `app_108jobs_db_views_comment` (workspace, features = ["full"])
- `app_108jobs_db_views_local_user` (workspace, features = ["full"])
- `app_108jobs_db_views_site` (workspace, features = ["full"])
- `app_108jobs_db_views_vote` (workspace, features = ["full"])
- `anyhow` (workspace)
- `chrono` (workspace)
- `serde` (workspace)
- `tracing` (workspace)

---

### Task 1: Create `crates/jobs/` and move post handlers

**Files:**
- Create: `crates/jobs/Cargo.toml`
- Create: `crates/jobs/src/lib.rs`
- Move: 11 files from `crates/api/api/src/post/` to `crates/jobs/src/`
- Move: 6 files from `crates/api/api_crud/src/post/` to `crates/jobs/src/` (prefix with `crud_` or use a `crud` sub-module to avoid name collisions with handler files)
- Delete source files from both old locations
- Edit: `crates/api/api/src/lib.rs` — remove `pub mod post;`
- Edit: `crates/api/api_crud/src/lib.rs` — remove `pub mod post;`
- Edit: root `Cargo.toml` — add `crates/jobs` member + workspace dep

**Name collision check:** Both `crates/api/api/src/post/` and `crates/api/api_crud/src/post/` have sub-files. In `crates/jobs/src/` they must not collide. Strategy: put handler files in `src/handlers/` and CRUD files in `src/crud/`, with `lib.rs` re-exporting both.

**Interfaces:**
- Produces: `cargo check -p app_108jobs_jobs` → 0 errors

- [ ] **Step 1: Investigate internal references**

```bash
grep -rn "super::\|crate::" crates/api/api/src/post/ crates/api/api_crud/src/post/
grep -rn "check_post_deleted_or_removed" crates/api/api_crud/src/
```

Fix any `super::` references found before moving.

- [ ] **Step 2: Create directory structure**

```bash
mkdir -p crates/jobs/src/handlers crates/jobs/src/crud
```

- [ ] **Step 3: Create `crates/jobs/Cargo.toml`**

Read actual imports from all source files to determine exact deps:
```bash
grep "^use app_108jobs\|^use actix\|^use chrono\|^use uuid\|^use serde\|^use anyhow\|^use tracing\|^use url" crates/api/api/src/post/*.rs crates/api/api_crud/src/post/*.rs | sed 's/::.*//' | sort -u
```

Then write `Cargo.toml` with those deps (all workspace, `features = ["full"]` on db_views).

- [ ] **Step 4: Create `crates/jobs/src/lib.rs`**

```rust
pub mod crud;
pub mod handlers;

// Re-export check_post_deleted_or_removed at crate root if used externally
pub use crud::check_post_deleted_or_removed;
```

Create `crates/jobs/src/handlers/mod.rs` listing all 11 handler modules.
Create `crates/jobs/src/crud/mod.rs` listing all 5 CRUD modules + helper.

- [ ] **Step 5: Copy handler files**

```bash
cp crates/api/api/src/post/feature.rs            crates/jobs/src/handlers/feature.rs
cp crates/api/api/src/post/get_link_metadata.rs  crates/jobs/src/handlers/get_link_metadata.rs
cp crates/api/api/src/post/hide.rs               crates/jobs/src/handlers/hide.rs
cp crates/api/api/src/post/like.rs               crates/jobs/src/handlers/like.rs
cp crates/api/api/src/post/list.rs               crates/jobs/src/handlers/list.rs
cp crates/api/api/src/post/list_post_likes.rs    crates/jobs/src/handlers/list_post_likes.rs
cp crates/api/api/src/post/lock.rs               crates/jobs/src/handlers/lock.rs
cp crates/api/api/src/post/mark_many_read.rs     crates/jobs/src/handlers/mark_many_read.rs
cp crates/api/api/src/post/mark_read.rs          crates/jobs/src/handlers/mark_read.rs
cp crates/api/api/src/post/save.rs               crates/jobs/src/handlers/save.rs
cp crates/api/api/src/post/update_notifications.rs crates/jobs/src/handlers/update_notifications.rs

cp crates/api/api_crud/src/post/create.rs  crates/jobs/src/crud/create.rs
cp crates/api/api_crud/src/post/read.rs    crates/jobs/src/crud/read.rs
cp crates/api/api_crud/src/post/update.rs  crates/jobs/src/crud/update.rs
cp crates/api/api_crud/src/post/delete.rs  crates/jobs/src/crud/delete.rs
cp crates/api/api_crud/src/post/remove.rs  crates/jobs/src/crud/remove.rs
cp crates/api/api_crud/src/post/mod.rs     crates/jobs/src/crud/mod.rs
```

- [ ] **Step 6: Fix any internal `super::` references in the copied files**

```bash
grep -rn "super::" crates/jobs/src/
```

For any `super::check_post_deleted_or_removed` → change to `crate::crud::check_post_deleted_or_removed`.

- [ ] **Step 7: Remove source files and update old crates**

```bash
rm crates/api/api/src/post/*.rs
rmdir crates/api/api/src/post/
rm crates/api/api_crud/src/post/*.rs
rmdir crates/api/api_crud/src/post/
```

Edit `crates/api/api/src/lib.rs` — remove `pub mod post;`
Edit `crates/api/api_crud/src/lib.rs` — remove `pub mod post;`

- [ ] **Step 8: Add to workspace**

In root `Cargo.toml` members: `"crates/jobs",`
In `[workspace.dependencies]`: `app_108jobs_jobs = { version = "=1.0.0-alpha.5", path = "./crates/jobs" }`

- [ ] **Step 9: Compile check**

```bash
cargo check -p app_108jobs_jobs 2>&1 | grep "^error" | head -10
```

Expected: 0 errors.

- [ ] **Step 10: Commit**

```bash
git add crates/jobs/ crates/api/ Cargo.toml Cargo.lock
git commit -m "refactor(phase-3): create crates/jobs/ with post/job handlers from api + api_crud"
```

---

### Task 2: Create `crates/proposals/` and move comment handlers

**Files:**
- Create: `crates/proposals/Cargo.toml`
- Create: `crates/proposals/src/lib.rs`
- Move: 5 files from `crates/api/api/src/comment/` + 5 files from `crates/api/api_crud/src/comment/`
- Delete source files from old locations
- Edit: `crates/api/api/src/lib.rs` — remove `pub mod comment;`
- Edit: `crates/api/api_crud/src/lib.rs` — remove `pub mod comment;`
- Edit: root `Cargo.toml` — add `crates/proposals` member + workspace dep

- [ ] **Step 1: Investigate internal references**

```bash
grep -rn "super::\|crate::" crates/api/api/src/comment/ crates/api/api_crud/src/comment/
```

- [ ] **Step 2: Create directory structure**

```bash
mkdir -p crates/proposals/src/handlers crates/proposals/src/crud
```

- [ ] **Step 3: Create `crates/proposals/Cargo.toml`**

Read actual imports from all comment source files, then write Cargo.toml.

- [ ] **Step 4: Create `crates/proposals/src/lib.rs`**

```rust
pub mod crud;
pub mod handlers;
```

Create `crates/proposals/src/handlers/mod.rs` listing 5 handler modules.
Create `crates/proposals/src/crud/mod.rs` listing 5 CRUD modules.

- [ ] **Step 5: Copy handler + CRUD files**

Copy 5 handler files to `src/handlers/` and 5 CRUD files to `src/crud/`.

- [ ] **Step 6: Fix `super::` references**

```bash
grep -rn "super::" crates/proposals/src/
```

Fix any found.

- [ ] **Step 7: Remove source files and update old crates**

```bash
rm crates/api/api/src/comment/*.rs
rmdir crates/api/api/src/comment/
rm crates/api/api_crud/src/comment/*.rs
rmdir crates/api/api_crud/src/comment/
```

Edit `crates/api/api/src/lib.rs` — remove `pub mod comment;`
Edit `crates/api/api_crud/src/lib.rs` — remove `pub mod comment;`

- [ ] **Step 8: Add to workspace**

In root `Cargo.toml` members: `"crates/proposals",`
In `[workspace.dependencies]`: `app_108jobs_proposals = { version = "=1.0.0-alpha.5", path = "./crates/proposals" }`

- [ ] **Step 9: Compile check**

```bash
cargo check -p app_108jobs_proposals 2>&1 | grep "^error" | head -10
```

Expected: 0 errors.

- [ ] **Step 10: Commit**

```bash
git add crates/proposals/ crates/api/ Cargo.toml Cargo.lock
git commit -m "refactor(phase-3): create crates/proposals/ with comment/proposal handlers from api + api_crud"
```

---

### Task 3: Wire routes + final gates

**Files:**
- Edit: root `Cargo.toml` `[dependencies]` — add `app_108jobs_jobs`, `app_108jobs_proposals`
- Edit: `src/api_routes.rs` — update all post and comment imports

- [ ] **Step 1: Find all current post/comment imports in api_routes.rs**

```bash
grep -n "post\|comment\|api_crud\|app_108jobs_api::" src/api_routes.rs | head -40
```

- [ ] **Step 2: Add new crates to root binary deps**

```toml
app_108jobs_jobs = { workspace = true }
app_108jobs_proposals = { workspace = true }
```

- [ ] **Step 3: Update imports in `src/api_routes.rs`**

Replace imports from `app_108jobs_api::post::*` and `app_108jobs_api_crud::post::*` with `app_108jobs_jobs::handlers::*` and `app_108jobs_jobs::crud::*`.

Replace imports from `app_108jobs_api::comment::*` and `app_108jobs_api_crud::comment::*` with `app_108jobs_proposals::handlers::*` and `app_108jobs_proposals::crud::*`.

Verify exact function names:
```bash
grep "^pub async fn\|^pub fn" crates/jobs/src/handlers/*.rs crates/jobs/src/crud/*.rs
grep "^pub async fn\|^pub fn" crates/proposals/src/handlers/*.rs crates/proposals/src/crud/*.rs
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
git commit -m "refactor(phase-3): wire app_108jobs_jobs + app_108jobs_proposals into api_routes"
```

---

## Self-Review

**Spec coverage:**
- ✅ `crates/jobs/` created as `app_108jobs_jobs`
- ✅ `crates/proposals/` created as `app_108jobs_proposals`
- ✅ Post handlers (11) + CRUD (6) moved from `api` + `api_crud`
- ✅ Comment handlers (5) + CRUD (5) moved from `api` + `api_crud`
- ✅ All files deleted from old locations
- ✅ `api/src/lib.rs` and `api_crud/src/lib.rs` updated
- ✅ `src/api_routes.rs` updated
- ✅ Both crates in workspace + main binary deps
- ✅ Gate: `cargo check --workspace` + 16/16 tests green
- ✅ DB rename NOT included (separate phase)

**No placeholders.** Compile-driven: any missed reference is a compiler error.
