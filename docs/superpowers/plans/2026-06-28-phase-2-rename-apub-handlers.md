# Phase 2: Rename Apub Route Handlers

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move the 5 live route-handler files out of `crates/apub/src/api/` into their natural homes in `crates/api/api/src/` (and `crates/api/api_utils/`), then delete the now-empty `crates/apub/` crate.

**Architecture:** The `crates/apub/` crate is a Lemmy artifact; the 5 handler files in it are pure 108Jobs business queries with no AP semantics left (Phase 1 removed all AP code). Moving them gives us: list_posts → `post/list.rs`, list_comments → `comment/list.rs`, search → `search.rs`, list_person_content → `local_user/person_content.rs`, read_category → `category/read.rs`. Three shared sort/listing helpers move to `api_utils`. After all moves the `apub` crate is empty and deleted.

**Tech Stack:** Rust, Cargo workspace, `cargo check`, `cargo nextest`

## Global Constraints

- **No behavior change.** The 3 HTTP-bound handlers (`search`, `list_posts`, `list_comments`) serve exactly the same URL paths after this move. The 2 feed-only handlers (`list_person_content`, `get_category`) continue to work for RSS.
- **No table drops, no migrations.** Only Rust code is moved/deleted.
- **Gate:** `cargo check --workspace` exits 0 (zero errors, zero warnings). `cargo nextest run -p app_108jobs_contract_tests` 16/16 pass.
- **Nightly fmt:** `cargo +nightly fmt` after all changes; `-- --check` verifies.
- **No new `#[allow(...)]` attributes.**
- Commit messages: `refactor(phase-2): <what moved>`.

## Context

Current state (all in `crates/apub/src/`):
```
lib.rs                     → pub mod api; (only this)
api/mod.rs                 → 5 mod declarations + 3 sort/listing utility fns
api/list_posts.rs          → list_posts() → GET /api/v4/post/list
api/list_comments.rs       → list_comments(), list_comments_slim() → GET /api/v4/comment/list
api/search.rs              → search() → GET /api/v4/search
api/list_person_content.rs → list_person_content() (feeds.rs only, not HTTP-routed directly)
api/read_category.rs       → get_category() (feeds.rs only)
```

Consumers:
- `src/api_routes.rs` line 212: `use app_108jobs_apub::api::{list_comments::list_comments, list_posts::list_posts, search::search};`
- `crates/routes/src/feeds.rs`: imports `list_person_content` and `get_category` from `app_108jobs_apub`

Target locations after Phase 2:
```
crates/api/api/src/post/list.rs               ← list_posts.rs
crates/api/api/src/comment/list.rs            ← list_comments.rs
crates/api/api/src/search.rs                  ← search.rs
crates/api/api/src/local_user/person_content.rs ← list_person_content.rs
crates/api/api/src/category/read.rs           ← read_category.rs
crates/api/api_utils/src/listing_defaults.rs  ← 3 utility fns from api/mod.rs
```

---

### Task 1: Move 3 utility functions to `api_utils`

**Files:**
- Create: `crates/api/api_utils/src/listing_defaults.rs`
- Modify: `crates/api/api_utils/src/lib.rs` — add `pub mod listing_defaults;`
- Modify: `crates/apub/src/api/mod.rs` — remove 3 utility fns (keep 5 mod declarations for now)

**Interfaces:**
- Produces: `app_108jobs_api_utils::listing_defaults::{listing_type_with_default, post_sort_type_with_default, comment_sort_type_with_default}` — same function signatures, just relocated. Later tasks use these.

- [ ] **Step 1: Read the 3 utility functions from `crates/apub/src/api/mod.rs`**

```bash
grep -n "fn listing_type_with_default\|fn post_sort_type_with_default\|fn comment_sort_type_with_default" crates/apub/src/api/mod.rs
```

Then read the full function bodies.

- [ ] **Step 2: Create `crates/api/api_utils/src/listing_defaults.rs`**

Copy the 3 functions verbatim — same signatures, same bodies. Add whatever imports they need (check what types `listing_type_with_default`, `post_sort_type_with_default`, `comment_sort_type_with_default` use: `ListingType`, `LocalUser`, `PostSortType`, `CommentSortType`, `LocalSite`, etc.). These types are in `app_108jobs_db_schema` and `app_108jobs_db_schema_file` which are already deps of `api_utils`.

Make the functions `pub`.

- [ ] **Step 3: Add `pub mod listing_defaults;` to `crates/api/api_utils/src/lib.rs`**

- [ ] **Step 4: Compile check `api_utils`**

```bash
cargo check -p app_108jobs_api_utils --all-features 2>&1 | grep "^error" | head -10
```

Expected: zero errors. Fix any import issues.

- [ ] **Step 5: Remove the 3 utility functions from `crates/apub/src/api/mod.rs`**

The mod.rs should now contain only the 5 `pub mod` declarations:
```rust
pub mod list_comments;
pub mod list_person_content;
pub mod list_posts;
pub mod read_category;
pub mod search;
```

- [ ] **Step 6: Compile check `apub`**

```bash
cargo check -p app_108jobs_apub 2>&1 | grep "^error" | head -10
```

At this point the apub handler files still import `use crate::api::listing_type_with_default` etc. — those will error. That's expected; Task 2 will fix them when the handlers move. As a temporary measure, update each apub handler that calls these functions to import from `app_108jobs_api_utils::listing_defaults` instead of `crate::api`.

**Files to update in `crates/apub/src/api/`:**
```bash
grep -rn "listing_type_with_default\|post_sort_type_with_default\|comment_sort_type_with_default" crates/apub/src/api/ --include="*.rs"
```

For each file that uses these, replace:
```rust
// OLD:
use crate::api::{listing_type_with_default, post_sort_type_with_default};
// NEW:
use app_108jobs_api_utils::listing_defaults::{listing_type_with_default, post_sort_type_with_default};
```

Also add `app_108jobs_api_utils = { workspace = true }` to `crates/apub/Cargo.toml` if not already present.

- [ ] **Step 7: Full workspace compile check**

```bash
cargo check --workspace 2>&1 | grep "^error" | head -10
```

Expected: zero errors.

- [ ] **Step 8: Commit**

```bash
cargo +nightly fmt
git add -A
git commit -m "refactor(phase-2): move listing/sort default helpers to api_utils"
```

---

### Task 2: Move 3 HTTP-bound handlers into `crates/api/api/`

**Files:**
- Move: `crates/apub/src/api/list_posts.rs` → `crates/api/api/src/post/list.rs`
- Move: `crates/apub/src/api/list_comments.rs` → `crates/api/api/src/comment/list.rs`
- Move: `crates/apub/src/api/search.rs` → `crates/api/api/src/search.rs`
- Modify: `crates/api/api/src/post/mod.rs` — add `pub mod list;`
- Modify: `crates/api/api/src/comment/mod.rs` — add `pub mod list;`
- Modify: `crates/api/api/src/lib.rs` — add `pub mod search;`
- Modify: `src/api_routes.rs` — update import paths
- Modify: `crates/apub/src/api/mod.rs` — remove `pub mod list_posts; pub mod list_comments; pub mod search;`
- Possibly modify: `crates/api/api/Cargo.toml` — add any missing deps

**Interfaces:**
- Consumes: `app_108jobs_api_utils::listing_defaults::*` (from Task 1)
- Produces: `app_108jobs_api::post::list::list_posts`, `app_108jobs_api::comment::list::{list_comments, list_comments_slim}`, `app_108jobs_api::search::search`

**Strategy:** Copy files to new locations, fix the `use crate::api::` import paths to not reference the `apub` crate, add `pub mod` declarations, update `src/api_routes.rs` imports, then run `cargo check --workspace` to find remaining breakage.

- [ ] **Step 1: Copy `list_posts.rs` to `crates/api/api/src/post/list.rs`**

```bash
cp crates/apub/src/api/list_posts.rs crates/api/api/src/post/list.rs
```

In the new file, fix any import that was relative to the `apub` crate:
- `use crate::api::listing_type_with_default` → `use app_108jobs_api_utils::listing_defaults::listing_type_with_default`
- `use crate::api::post_sort_type_with_default` → `use app_108jobs_api_utils::listing_defaults::post_sort_type_with_default`

Make the handler function(s) `pub`.

- [ ] **Step 2: Copy `list_comments.rs` to `crates/api/api/src/comment/list.rs`**

```bash
cp crates/apub/src/api/list_comments.rs crates/api/api/src/comment/list.rs
```

Fix imports: `use crate::api::comment_sort_type_with_default` → `use app_108jobs_api_utils::listing_defaults::comment_sort_type_with_default`

Make the handler functions `pub`.

- [ ] **Step 3: Copy `search.rs` to `crates/api/api/src/search.rs`**

```bash
cp crates/apub/src/api/search.rs crates/api/api/src/search.rs
```

Fix any `crate::api::*` imports (search.rs may not have any — check with `grep "crate::api" crates/apub/src/api/search.rs`).

Make the handler function `pub`.

- [ ] **Step 4: Register new modules in `crates/api/api/src/`**

In `crates/api/api/src/post/mod.rs`, add:
```rust
pub mod list;
```

In `crates/api/api/src/comment/mod.rs`, add:
```rust
pub mod list;
```

In `crates/api/api/src/lib.rs`, add:
```rust
pub mod search;
```

- [ ] **Step 5: Update `src/api_routes.rs` import**

Change:
```rust
use app_108jobs_apub::api::{list_comments::list_comments, list_posts::list_posts, search::search};
```

To:
```rust
use app_108jobs_api::{comment::list::list_comments, post::list::list_posts, search::search};
```

(Verify the exact crate name of `crates/api/api/` via `grep "^name" crates/api/api/Cargo.toml`.)

- [ ] **Step 6: Remove moved modules from `crates/apub/src/api/mod.rs`**

Remove:
```rust
pub mod list_comments;
pub mod list_posts;
pub mod search;
```

`mod.rs` should now only have:
```rust
pub mod list_person_content;
pub mod read_category;
```

- [ ] **Step 7: Run `cargo check --workspace` to find breakage**

```bash
cargo check --workspace 2>&1 | grep "^error" | head -20
```

Fix errors:
- Missing deps in `crates/api/api/Cargo.toml`: run `grep "^use " crates/api/api/src/post/list.rs | grep -v "app_108jobs\|actix\|serde"` to find external deps (`strum`, `url`, etc.) and add them to Cargo.toml from `crates/apub/Cargo.toml`
- Any remaining import path issues

- [ ] **Step 8: Delete original files from `crates/apub/`**

```bash
rm crates/apub/src/api/list_posts.rs
rm crates/apub/src/api/list_comments.rs
rm crates/apub/src/api/search.rs
```

- [ ] **Step 9: Final workspace compile check**

```bash
cargo check --workspace 2>&1 | grep "^error" | head -10
```

Expected: zero errors.

- [ ] **Step 10: Run contract tests**

```bash
cargo nextest run -p app_108jobs_contract_tests 2>&1 | tail -3
```

Expected: 16/16 pass.

- [ ] **Step 11: Nightly fmt + commit**

```bash
cargo +nightly fmt
git add -A
git commit -m "refactor(phase-2): move list_posts, list_comments, search into api crate"
```

---

### Task 3: Move 2 feed-only handlers, delete `crates/apub/`

**Files:**
- Move: `crates/apub/src/api/list_person_content.rs` → `crates/api/api/src/local_user/person_content.rs`
- Move: `crates/apub/src/api/read_category.rs` → `crates/api/api/src/category/read.rs`
- Modify: `crates/api/api/src/local_user/mod.rs` — add `pub mod person_content;`
- Modify: `crates/api/api/src/category/mod.rs` — add `pub mod read;`
- Modify: `crates/routes/src/feeds.rs` — update imports
- Modify: `crates/routes/Cargo.toml` — remove `app_108jobs_apub` dep, add `app_108jobs_api` if not present
- Delete: `crates/apub/` (entire directory — now empty of logic)
- Modify: root `Cargo.toml` — remove `apub` from workspace members and workspace.dependencies
- Modify: any remaining `Cargo.toml` that has `app_108jobs_apub` as a dep

**Interfaces:**
- Produces: `app_108jobs_api::local_user::person_content::list_person_content`, `app_108jobs_api::category::read::get_category`. `crates/apub/` crate fully gone.

- [ ] **Step 1: Copy `list_person_content.rs`**

```bash
cp crates/apub/src/api/list_person_content.rs crates/api/api/src/local_user/person_content.rs
```

Fix any `crate::api::*` imports (check: `grep "crate::api" crates/apub/src/api/list_person_content.rs`).

Make the handler function `pub`.

- [ ] **Step 2: Copy `read_category.rs`**

```bash
cp crates/apub/src/api/read_category.rs crates/api/api/src/category/read.rs
```

Fix any `crate::api::*` imports.

Make the handler function `pub`.

- [ ] **Step 3: Register new modules**

In `crates/api/api/src/local_user/mod.rs`, add:
```rust
pub mod person_content;
```

In `crates/api/api/src/category/mod.rs`, add:
```rust
pub mod read;
```

- [ ] **Step 4: Check where `crates/routes/src/feeds.rs` imports from apub**

```bash
grep -n "app_108jobs_apub\|apub" crates/routes/src/feeds.rs | head -20
```

Update the import(s) to reference the new location:
```rust
// OLD (example):
use app_108jobs_apub::api::{list_person_content::list_person_content, read_category::get_category};
// NEW:
use app_108jobs_api::{local_user::person_content::list_person_content, category::read::get_category};
```

- [ ] **Step 5: Find all remaining `app_108jobs_apub` references**

```bash
grep -rn "app_108jobs_apub" . --include="*.toml" --include="*.rs" | grep -v "target/" | grep -v ".git/"
```

Update every reference found:
- In `Cargo.toml` files: remove `app_108jobs_apub` dep lines, add `app_108jobs_api` if needed
- In `.rs` files: update `use app_108jobs_apub::*` to the new paths

- [ ] **Step 6: Remove moved files from `crates/apub/`**

```bash
rm crates/apub/src/api/list_person_content.rs
rm crates/apub/src/api/read_category.rs
rm crates/apub/src/api/mod.rs
rm crates/apub/src/lib.rs
```

- [ ] **Step 7: Delete the empty `crates/apub/` directory**

```bash
rm -rf crates/apub/
```

- [ ] **Step 8: Remove `apub` from workspace root `Cargo.toml`**

In root `Cargo.toml`, remove from `members`:
```toml
    "crates/apub",
```

Remove from `[workspace.dependencies]`:
```toml
app_108jobs_apub = { version = "=1.0.0-alpha.5", path = "./crates/apub" }
```

- [ ] **Step 9: Full workspace compile check**

```bash
cargo check --workspace 2>&1 | grep "^error" | head -20
```

Fix any remaining import errors using the compiler output as a guide.

- [ ] **Step 10: Run contract tests**

```bash
cargo nextest run -p app_108jobs_contract_tests 2>&1 | tail -3
```

Expected: 16/16 pass.

- [ ] **Step 11: Nightly fmt + clippy + commit**

```bash
cargo +nightly fmt
cargo +nightly fmt -- --check
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | grep "^error\|^warning" | head -20
git add -A
git commit -m "refactor(phase-2): move feed handlers, delete apub crate"
```

---

## Self-Review

**Spec coverage:**
- ✅ All 5 handlers moved out of `crates/apub/`
- ✅ 3 utility functions moved to `api_utils`
- ✅ `list_posts` → `post/list.rs` (Task 2)
- ✅ `list_comments` → `comment/list.rs` (Task 2)
- ✅ `search` → `search.rs` (Task 2)
- ✅ `list_person_content` → `local_user/person_content.rs` (Task 3)
- ✅ `get_category` → `category/read.rs` (Task 3)
- ✅ `crates/apub/` deleted (Task 3)
- ✅ All HTTP routes unchanged (`/api/v4/search`, `/api/v4/post/list`, `/api/v4/comment/list`)
- ✅ RSS feed handlers still work
- ✅ Gate: `cargo check --workspace` + 16/16 contract tests green

**No placeholders.** Compile-driven strategy ensures no reference is missed.
