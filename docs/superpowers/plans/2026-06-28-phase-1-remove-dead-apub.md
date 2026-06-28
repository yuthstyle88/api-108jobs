# Phase 1: Remove Proven-Dead ActivityPub Code

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Strip the dead ActivityPub / Lemmy federation scaffolding from `api-108jobs`. Code only — no table drops. After this phase the workspace still compiles and all contract tests pass.

**Architecture:** Three orthogonal deletions (apub_objects crate, ActivityChannel mechanism, activity ORM), two small cleanups (dead task, CLI flag). Each is self-contained: do them one by one, compile-check after each.

**Tech Stack:** Rust, Cargo workspace, `cargo check`, `cargo nextest`

## Global Constraints

- **Tables stay.** `sent_activity` and `received_activity` in `crates/db_schema_file/src/schema.rs` are NOT touched. Only Rust code is removed.
- **`crates/apub/` stays.** The `apub` crate (5 live route handlers) is NOT deleted — that happens in Phase 2. Only `crates/apub_objects/` is deleted and the `assets/` test-fixture folder.
- **No behavior change.** The live API routes (`/search`, `/posts/list`, `/comments/list`, etc.) continue to work. Only dead-channel calls are removed.
- **Gate:** `cargo check --workspace` exits 0. `cargo nextest run -p app_108jobs_contract_tests` exits 0 (all 16 tests pass).
- **Nightly fmt:** `cargo +nightly fmt` after all changes. Check with `cargo +nightly fmt -- --check`.
- **No new `#[allow(...)]` attributes.**
- Commit messages: `refactor(phase-1): <what was removed>`.

---

### Task 1: Inline `ApubCategory` and delete `crates/apub_objects/`

**Files:**
- Modify: `crates/apub/src/api/list_comments.rs` — inline `Category::read_from_name`
- Modify: `crates/apub/Cargo.toml` — remove `app_108jobs_apub_objects` dep
- Modify: `crates/apub/src/lib.rs` — remove `VerifyUrlData`, `FEDERATION_HTTP_FETCH_LIMIT`, `pub mod fetcher`
- Delete: `crates/apub/src/fetcher/mod.rs` (and the `fetcher/` directory)
- Delete: `crates/apub_objects/` (whole directory, ~20 Rust files)
- Modify: `Cargo.toml` (workspace root) — remove `apub_objects` from members + deps

**Interfaces:**
- Consumes: `Category::read_from_name` from `app_108jobs_db_schema::impls::category`
- Produces: `apub` crate compiles without `apub_objects` dep; `apub_objects` crate gone

- [ ] **Step 1: Inline `ApubCategory` in `list_comments.rs`**

Current `crates/apub/src/api/list_comments.rs` (lines 1-7, 34-36):
```rust
// OLD - remove these two lines:
use crate::{api::listing_type_with_default, fetcher::resolve_ap_identifier};
use app_108jobs_apub_objects::objects::category::ApubCategory;

// OLD - change this:
Some(resolve_ap_identifier::<ApubCategory, Category>(name, &context, true).await?).map(|c| c.id)

// NEW - replace with:
Category::read_from_name(&mut context.pool(), name, true).await?.map(|c| c.id)
```

After the change, `list_comments.rs` imports should keep:
```rust
use crate::api::listing_type_with_default;
use app_108jobs_db_schema::{
  newtypes::PaginationCursor,
  source::{category::Category, comment::Comment},
  traits::{Crud, PaginationCursorBuilder},
};
```

Also add `traits::ApubActor` if `Category::read_from_name` requires it (check via `cargo check`). The `ApubActor` trait is in scope via `use app_108jobs_db_schema::traits::ApubActor` — add it only if the compiler says it's needed.

- [ ] **Step 2: Check if `fetcher/mod.rs` is now dead**

Run `cargo check -p app_108jobs_apub 2>&1 | grep unused`. If `resolve_ap_identifier` is now unused (nothing else in the `apub` crate calls it), delete the fetcher module:

```bash
rm -rf crates/apub/src/fetcher/
```

Then remove `pub mod fetcher;` from `crates/apub/src/lib.rs`.

- [ ] **Step 3: Remove dead constants from `crates/apub/src/lib.rs`**

After removing `pub mod fetcher`, check if `FEDERATION_HTTP_FETCH_LIMIT` and `VerifyUrlData` are used anywhere:

```bash
grep -rn "FEDERATION_HTTP_FETCH_LIMIT\|VerifyUrlData" crates/ --include="*.rs" | grep -v "target/"
```

If they appear only in `crates/apub/src/lib.rs` itself (not imported elsewhere), remove them from `lib.rs`. If they're imported by other crates, leave them.

Expected: `FEDERATION_HTTP_FETCH_LIMIT` is unused, `VerifyUrlData` is unused — both should be removed.

After cleanup, `crates/apub/src/lib.rs` should look like:
```rust
pub mod api;
```

- [ ] **Step 4: Remove `app_108jobs_apub_objects` from `crates/apub/Cargo.toml`**

Find and remove the line:
```toml
app_108jobs_apub_objects = { workspace = true }
```

- [ ] **Step 5: Delete `crates/apub_objects/` directory**

```bash
rm -rf crates/apub_objects/
```

- [ ] **Step 6: Remove `apub_objects` from workspace root `Cargo.toml`**

In `/Users/koeyl/108-ecosystem/108jobs/api-108jobs/Cargo.toml`:

Remove from `members = [`:
```toml
    "crates/apub_objects",
```

Remove from `[workspace.dependencies]`:
```toml
app_108jobs_apub_objects = { version = "=1.0.0-alpha.5", path = "./crates/apub_objects" }
```

- [ ] **Step 7: Compile check**

```bash
cargo check -p app_108jobs_apub 2>&1 | tail -5
```

Expected: `Finished` with no errors. Fix any remaining import errors by reading the compiler output.

```bash
cargo +nightly fmt -p app_108jobs_apub
```

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "refactor(phase-1): inline ApubCategory, delete apub_objects crate"
```

---

### Task 2: Remove `ActivityChannel` mechanism and clean up 26 callers

**Files:**
- Delete/gut: `crates/api/api_utils/src/send_activity.rs`
- Modify: `crates/api/api_utils/src/request.rs` — remove `send_activity` param from `generate_post_link_metadata`
- Modify: 24 handler files — remove `ActivityChannel::submit_activity(...)` calls and imports

**Interfaces:**
- Produces: `send_activity.rs` gone; `generate_post_link_metadata` takes 3 params (no `send_activity`); no handler calls `ActivityChannel::submit_activity` or uses `SendActivityData`

**Strategy:** Delete `send_activity.rs`, then use `cargo check` output to find every file that fails to compile because of missing `ActivityChannel`/`SendActivityData` imports. Fix each file by removing the dead import and the dead call.

- [ ] **Step 1: Delete `send_activity.rs`**

```bash
rm crates/api/api_utils/src/send_activity.rs
```

Remove `pub mod send_activity;` from `crates/api/api_utils/src/lib.rs`.

- [ ] **Step 2: Fix `request.rs` — remove `send_activity` param**

`crates/api/api_utils/src/request.rs` currently:
```rust
use crate::send_activity::{ActivityChannel, SendActivityData};   // remove
...
pub async fn generate_post_link_metadata(
  post: Post,
  custom_thumbnail: Option<Url>,
  send_activity: impl FnOnce(Post) -> Option<SendActivityData> + Send + 'static,  // remove
  context: Data<FastJobContext>,
) -> FastJobResult<()> {
  ...
  if let Some(send_activity) = send_activity(updated_post) {   // remove these 3 lines
    ActivityChannel::submit_activity(send_activity, &context)?;
  }
  Ok(())
}
```

After:
```rust
pub async fn generate_post_link_metadata(
  post: Post,
  custom_thumbnail: Option<Url>,
  context: Data<FastJobContext>,
) -> FastJobResult<()> {
  ...
  // (remove the send_activity call at the end; keep just `Ok(())`)
}
```

Remove the `use crate::send_activity::{ActivityChannel, SendActivityData};` import from `request.rs`.

- [ ] **Step 3: Run `cargo check --workspace` to get the full error list**

```bash
cargo check --workspace 2>&1 | grep "^error" | head -40
```

Each error will point to a file and line. The errors will be:
- `use ... send_activity::{ActivityChannel, SendActivityData}` import not found
- `ActivityChannel::submit_activity(...)` call on removed type
- `SendActivityData::...` variant usage
- In `post/update.rs`: extra argument passed to `generate_post_link_metadata` (now takes 3 args)

- [ ] **Step 4: Fix each failing file**

For each file the compiler reports, apply this pattern:

**Pattern A — file calls `ActivityChannel::submit_activity(data, &context)?;`:**
1. Remove the `use ... send_activity::{ActivityChannel, SendActivityData};` import line
2. Remove the `ActivityChannel::submit_activity(...)` call and its argument construction

Example — `crates/api/api_crud/src/post/delete.rs`:
```rust
// Remove this import line:
  send_activity::{ActivityChannel, SendActivityData},
// Remove this call and its arguments:
  ActivityChannel::submit_activity(
    SendActivityData::DeletePost(post, local_user_view.person.clone(), data.0),
    &context,
  )?;
```

**Pattern B — file passes lambda to `generate_post_link_metadata`:**
`crates/api/api_crud/src/post/update.rs` — remove the `send_activity` argument (3rd positional arg):
```rust
// OLD:
generate_post_link_metadata(
  updated_post.clone(),
  custom_thumbnail.flatten().map(Into::into),
  |post| Some(SendActivityData::CreatePost(post)),   // remove this line
  context.clone(),
)
// NEW:
generate_post_link_metadata(
  updated_post.clone(),
  custom_thumbnail.flatten().map(Into::into),
  context.clone(),
)
```
Also remove `use ... send_activity::SendActivityData;` from `post/update.rs`.

The complete list of files with submit_activity calls (26 total):
1. `crates/api/api/src/comment/distinguish.rs`
2. `crates/api/api/src/comment/like.rs`
3. `crates/api/api/src/local_user/ban_person.rs`
4. `crates/api/api/src/post/feature.rs`
5. `crates/api/api/src/post/like.rs`
6. `crates/api/api/src/post/lock.rs`
7. `crates/api/api/src/reports/category_report/create.rs`
8. `crates/api/api/src/reports/category_report/resolve.rs`
9. `crates/api/api/src/reports/comment_report/create.rs`
10. `crates/api/api/src/reports/comment_report/resolve.rs`
11. `crates/api/api/src/reports/post_report/create.rs`
12. `crates/api/api/src/reports/post_report/resolve.rs`
13. `crates/api/api/src/site/purge/comment.rs`
14. `crates/api/api/src/site/purge/person.rs`
15. `crates/api/api/src/site/purge/post.rs`
16. `crates/api/api_crud/src/category/delete.rs`
17. `crates/api/api_crud/src/category/update.rs`
18. `crates/api/api_crud/src/comment/create.rs`
19. `crates/api/api_crud/src/comment/delete.rs`
20. `crates/api/api_crud/src/comment/remove.rs`
21. `crates/api/api_crud/src/comment/update.rs`
22. `crates/api/api_crud/src/post/delete.rs`
23. `crates/api/api_crud/src/post/remove.rs`
24. `crates/api/api_crud/src/post/update.rs`
25. `crates/api/api_crud/src/user/delete.rs`
26. `crates/api/api_utils/src/request.rs` (already done in Step 2)

- [ ] **Step 5: Final compile check**

```bash
cargo check --workspace 2>&1 | grep "^error" | head -10
```

Expected: zero errors.

- [ ] **Step 6: Nightly fmt + clippy**

```bash
cargo +nightly fmt
cargo +nightly fmt -- --check
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | grep "^error\|^warning" | head -20
```

Fix any clippy warnings introduced by this task.

- [ ] **Step 7: Run contract tests**

```bash
cargo nextest run -p app_108jobs_contract_tests 2>&1 | tail -5
```

Expected: 16/16 pass.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "refactor(phase-1): remove ActivityChannel dead federation channel and 26 callers"
```

---

### Task 3: Remove activity ORM, dead scheduled task, and CLI flag

**Files:**
- Delete: `crates/db_schema/src/source/activity.rs`
- Delete: `crates/db_schema/src/impls/activity.rs`
- Modify: `crates/db_schema/src/source/mod.rs` — remove `pub mod activity;`
- Modify: `crates/db_schema/src/impls/mod.rs` — remove `pub mod activity;`
- Modify: `crates/routes/src/utils/scheduled_tasks.rs` — remove `_active_counts()`
- Modify: `src/lib.rs` — remove `disable_activity_sending` CLI flag
- Delete: `crates/apub/assets/` test fixture directory

**Interfaces:**
- Produces: `activity.rs` ORM code gone; no `ActivitySendTargets`/`SentActivity`/`ReceivedActivity` in scope; scheduled_tasks.rs has no dead functions; CLI struct has no `disable_activity_sending` field

- [ ] **Step 1: Delete activity ORM files**

```bash
rm crates/db_schema/src/source/activity.rs
rm crates/db_schema/src/impls/activity.rs
```

- [ ] **Step 2: Remove `pub mod activity` from mod.rs files**

In `crates/db_schema/src/source/mod.rs`:
Remove the line: `pub mod activity;`

In `crates/db_schema/src/impls/mod.rs`:
Remove the line: `pub mod activity;`

- [ ] **Step 3: Compile check for activity references**

```bash
cargo check --workspace 2>&1 | grep "^error" | head -20
```

If anything still references `ActivitySendTargets`, `SentActivity`, `ReceivedActivity`, `SentActivityForm` — those references should only be in `send_activity.rs` (already deleted) and `activity.rs` (now deleted). Any remaining errors indicate an unexpected reference — investigate and remove.

The `sent_activity` and `received_activity` in `schema.rs` (Diesel table macros) are NOT touched — they stay.

- [ ] **Step 4: Remove `_active_counts()` from scheduled_tasks.rs**

In `crates/routes/src/utils/scheduled_tasks.rs`, find and remove the function `_active_counts()`.

Check for it:
```bash
grep -n "_active_counts\|fn _active_counts" crates/routes/src/utils/scheduled_tasks.rs
```

Remove the entire function body (typically ~30 lines of SQL string execution). Do NOT remove `_delete_expired_captcha_answers()` — that is out of scope for Phase 1.

- [ ] **Step 5: Remove `disable_activity_sending` CLI flag**

In `src/lib.rs`:

```bash
grep -n "disable_activity_sending" src/lib.rs
```

Remove the `/// ...` doc comment and `disable_activity_sending: bool` field from the `CmdArgs` struct. There should be no usages of `args.disable_activity_sending` (it was never read — confirm with `grep -rn "disable_activity_sending" src/`).

- [ ] **Step 6: Delete test fixture assets**

```bash
rm -rf crates/apub/assets/
```

- [ ] **Step 7: Final compile check**

```bash
cargo check --workspace 2>&1 | grep "^error" | head -10
```

Expected: zero errors.

- [ ] **Step 8: Run contract tests**

```bash
cargo nextest run -p app_108jobs_contract_tests 2>&1 | tail -5
```

Expected: 16/16 pass.

- [ ] **Step 9: Nightly fmt + clippy**

```bash
cargo +nightly fmt
cargo +nightly fmt -- --check
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | grep "^error\|^warning" | head -20
```

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "refactor(phase-1): remove activity ORM, dead task, CLI flag, test fixtures"
```

---

## Self-Review

**Spec coverage:**
- ✅ `apub_objects` crate deleted (Task 1)
- ✅ `ApubCategory` import inlined to direct `Category::read_from_name` call (Task 1)
- ✅ `ActivityChannel` / `MATCH_OUTGOING_ACTIVITIES` / `SendActivityData` deleted (Task 2)
- ✅ All 25 `submit_activity` call sites cleaned (Task 2)
- ✅ `generate_post_link_metadata` `send_activity` param removed (Task 2)
- ✅ `activity.rs` source + impls deleted (Task 3)
- ✅ `_active_counts` dead task removed (Task 3)
- ✅ `disable_activity_sending` CLI flag removed (Task 3)
- ✅ `crates/apub/assets/` test fixtures deleted (Task 3)
- ✅ `crates/apub/` crate retained (routes still live; Phase 2 renames them)
- ✅ `sent_activity`/`received_activity` tables in `schema.rs` untouched
- ✅ Gate: `cargo check --workspace` + 16 contract tests pass

**No placeholders.** All code patterns are shown. Compile-driven approach for Task 2 ensures no call site is missed.
