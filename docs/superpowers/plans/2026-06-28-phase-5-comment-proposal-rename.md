# Phase 5 — comment → proposal Full Rename

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement task-by-task.

**Goal:** Rename the `comment` concept to `proposal` across the entire stack: DB tables/columns/indexes/sequences/enums, replaceable SQL schema, Rust source types, crate names, API route paths.

**Architecture:** Four tasks — (1) DB migration, (2) schema.rs regeneration + replaceable schema + Rust enums, (3) mass Rust rename in db/db_views crates, (4) mass Rust rename in api/handler crates + route paths + final gates.

**Tech Stack:** Rust/Actix/Diesel/PostgreSQL, Cargo workspace

## Global Constraints

- Branch: `phase-5/comment-to-proposal`. Never touch main.
- `cargo check --workspace` → 0 errors after every task.
- `cargo nextest run -p app_108jobs_contract_tests` → 16/16 after final task.
- `cargo +nightly fmt` before every commit.
- Local DB: `postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1`
- **ap_id columns in comment/post/person already dropped in Phase 4** — do NOT reference or recreate them.
- **`comment_change_values()` trigger already removed in Phase 4** — do NOT recreate it.
- `local_user.collapse_bot_comments` — rename to `collapse_bot_proposals`
- `delivery_rider_rating.comment` (free text) — LEAVE AS-IS (user-visible label, not the concept rename)
- `user_review.comment` (free text) — LEAVE AS-IS

---

## Phase 4 Items Already Done (DO NOT REDO)

- `comment.ap_id` column — dropped in Phase 4 migration
- `comment.local` column — dropped in Phase 4 migration
- `r.comment_change_values()` trigger — removed from triggers.sql in Phase 4

---

### Task 1: DB Migration

**Files:**
- Create: `migrations/2026-06-28-130000-0000_rename_comment_to_proposal/up.sql`
- Create: `migrations/2026-06-28-130000-0000_rename_comment_to_proposal/down.sql`

**What to do — run in this exact order (FK-safe):**

```sql
-- up.sql

-- 1. Drop CHECK constraints on combined tables (they reference comment column names)
ALTER TABLE modlog_combined DROP CONSTRAINT IF EXISTS modlog_combined_check;
ALTER TABLE inbox_combined DROP CONSTRAINT IF EXISTS inbox_combined_check;
ALTER TABLE report_combined DROP CONSTRAINT IF EXISTS report_combined_check;
ALTER TABLE search_combined DROP CONSTRAINT IF EXISTS search_combined_check;
ALTER TABLE person_content_combined DROP CONSTRAINT IF EXISTS person_content_combined_check;
ALTER TABLE person_saved_combined DROP CONSTRAINT IF EXISTS person_saved_combined_check;
ALTER TABLE person_liked_combined DROP CONSTRAINT IF EXISTS person_liked_combined_check;

-- 2. Rename leaf tables first (no tables FK into them)
ALTER TABLE comment_actions RENAME TO proposal_actions;
ALTER TABLE comment_reply RENAME TO proposal_reply;
ALTER TABLE comment_report RENAME TO proposal_report;
ALTER TABLE admin_purge_comment RENAME TO admin_purge_proposal;
ALTER TABLE mod_remove_comment RENAME TO mod_remove_proposal;
ALTER TABLE person_comment_mention RENAME TO person_proposal_mention;

-- 3. Rename main table last
ALTER TABLE comment RENAME TO proposal;

-- 4. Rename FK columns in other tables
ALTER TABLE modlog_combined RENAME COLUMN admin_purge_comment_id TO admin_purge_proposal_id;
ALTER TABLE modlog_combined RENAME COLUMN mod_remove_comment_id TO mod_remove_proposal_id;
ALTER TABLE report_combined RENAME COLUMN comment_report_id TO proposal_report_id;
ALTER TABLE inbox_combined RENAME COLUMN comment_reply_id TO proposal_reply_id;
ALTER TABLE inbox_combined RENAME COLUMN person_comment_mention_id TO person_proposal_mention_id;
ALTER TABLE person_content_combined RENAME COLUMN comment_id TO proposal_id;
ALTER TABLE person_saved_combined RENAME COLUMN comment_id TO proposal_id;
ALTER TABLE person_liked_combined RENAME COLUMN comment_id TO proposal_id;
ALTER TABLE search_combined RENAME COLUMN comment_id TO proposal_id;

-- 5. Rename business table columns
ALTER TABLE billing RENAME COLUMN comment_id TO proposal_id;
ALTER TABLE chat_room RENAME COLUMN current_comment_id TO current_proposal_id;

-- Check if delivery_details has linked_comment_id:
-- ALTER TABLE delivery_details RENAME COLUMN linked_comment_id TO linked_proposal_id;

-- 6. Rename person stat columns
ALTER TABLE person RENAME COLUMN comment_count TO proposal_count;
ALTER TABLE person RENAME COLUMN comment_score TO proposal_score;

-- 7. Rename post counter columns
ALTER TABLE post RENAME COLUMN comments TO proposals;
ALTER TABLE post RENAME COLUMN newest_comment_time_necro_at TO newest_proposal_time_necro_at;
ALTER TABLE post RENAME COLUMN newest_comment_time_at TO newest_proposal_time_at;

-- 8. Rename category/site stat columns
ALTER TABLE category RENAME COLUMN comments TO proposals;
ALTER TABLE local_site RENAME COLUMN comments TO proposals;

-- 9. Rename rate limit columns
ALTER TABLE local_site_rate_limit RENAME COLUMN comment_max_requests TO proposal_max_requests;
ALTER TABLE local_site_rate_limit RENAME COLUMN comment_interval_seconds TO proposal_interval_seconds;

-- 10. Rename post_actions columns
ALTER TABLE post_actions RENAME COLUMN read_comments_at TO read_proposals_at;
ALTER TABLE post_actions RENAME COLUMN read_comments_amount TO read_proposals_amount;

-- 11. Rename local_user column
ALTER TABLE local_user RENAME COLUMN default_comment_sort_type TO default_proposal_sort_type;
ALTER TABLE local_user RENAME COLUMN collapse_bot_comments TO collapse_bot_proposals;

-- 12. Rename local_site column
ALTER TABLE local_site RENAME COLUMN default_comment_sort_type TO default_proposal_sort_type;

-- 13. Rename indexes (best-effort: IF EXISTS handles already-renamed indexes)
ALTER INDEX IF EXISTS idx_comment_creator RENAME TO idx_proposal_creator;
ALTER INDEX IF EXISTS idx_comment_post RENAME TO idx_proposal_post;
ALTER INDEX IF EXISTS idx_comment_published RENAME TO idx_proposal_published;
ALTER INDEX IF EXISTS idx_comment_language RENAME TO idx_proposal_language;
ALTER INDEX IF EXISTS idx_comment_content_trigram RENAME TO idx_proposal_content_trigram;
ALTER INDEX IF EXISTS idx_comment_controversy RENAME TO idx_proposal_controversy;
ALTER INDEX IF EXISTS idx_comment_hot RENAME TO idx_proposal_hot;
ALTER INDEX IF EXISTS idx_comment_nonzero_hotrank RENAME TO idx_proposal_nonzero_hotrank;
ALTER INDEX IF EXISTS idx_comment_score RENAME TO idx_proposal_score;
ALTER INDEX IF EXISTS idx_comment_actions_liked_not_null RENAME TO idx_proposal_actions_liked_not_null;
ALTER INDEX IF EXISTS idx_comment_actions_saved_not_null RENAME TO idx_proposal_actions_saved_not_null;
ALTER INDEX IF EXISTS idx_comment_actions_like_score RENAME TO idx_proposal_actions_like_score;
ALTER INDEX IF EXISTS idx_comment_reply_comment RENAME TO idx_proposal_reply_proposal;
ALTER INDEX IF EXISTS idx_comment_reply_recipient RENAME TO idx_proposal_reply_recipient;
ALTER INDEX IF EXISTS idx_comment_reply_published RENAME TO idx_proposal_reply_published;
ALTER INDEX IF EXISTS idx_comment_report_published RENAME TO idx_proposal_report_published;
ALTER INDEX IF EXISTS idx_chat_room_current_comment_id RENAME TO idx_chat_room_current_proposal_id;

-- 14. Rename sequences
ALTER SEQUENCE IF EXISTS comment_id_seq RENAME TO proposal_id_seq;
ALTER SEQUENCE IF EXISTS comment_reply_id_seq RENAME TO proposal_reply_id_seq;
ALTER SEQUENCE IF EXISTS comment_report_id_seq RENAME TO proposal_report_id_seq;
ALTER SEQUENCE IF EXISTS person_comment_mention_id_seq RENAME TO person_proposal_mention_id_seq;

-- 15. Rename enum type
ALTER TYPE comment_sort_type_enum RENAME TO proposal_sort_type_enum;

-- 16. Re-add CHECK constraints with updated column names
-- (Read current CHECK bodies from pg_constraint before writing these)
-- These must be re-added after column renames. Exact SQL depends on current constraint bodies.
-- Run this to discover current bodies BEFORE running the migration:
-- SELECT conname, pg_get_constraintdef(oid) FROM pg_constraint WHERE conrelid IN (
--   'modlog_combined'::regclass, 'inbox_combined'::regclass, 'report_combined'::regclass,
--   'search_combined'::regclass, 'person_content_combined'::regclass,
--   'person_saved_combined'::regclass, 'person_liked_combined'::regclass
-- ) AND contype = 'c';
```

**Before writing the migration**, query the DB to discover the exact CHECK constraint bodies:
```bash
psql "postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1" -c "
SELECT conrelid::regclass AS table_name, conname, pg_get_constraintdef(oid) AS definition
FROM pg_constraint
WHERE contype = 'c'
AND conrelid IN (
  'modlog_combined'::regclass, 'inbox_combined'::regclass, 'report_combined'::regclass,
  'search_combined'::regclass, 'person_content_combined'::regclass,
  'person_saved_combined'::regclass, 'person_liked_combined'::regclass
)
ORDER BY table_name, conname;"
```

Also verify which columns actually exist before renaming:
```bash
psql "postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1" -c "
SELECT table_name, column_name FROM information_schema.columns
WHERE table_schema='public'
AND column_name IN ('linked_comment_id','current_comment_id','comment_id','comment_count','comment_score','comments','comment_reply_id','person_comment_mention_id','comment_report_id','admin_purge_comment_id','mod_remove_comment_id','read_comments_at','read_comments_amount','newest_comment_time_necro_at','newest_comment_time_at','default_comment_sort_type','collapse_bot_comments','comment_max_requests','comment_interval_seconds')
ORDER BY table_name, column_name;"
```

Run migration:
```bash
export DATABASE_URL="postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1"
diesel migration run --migration-dir migrations/
```

- [ ] Query DB for CHECK constraint bodies
- [ ] Query DB for actual column existence (skip missing columns in up.sql)
- [ ] Write up.sql with correct CHECK constraint re-adds
- [ ] Write down.sql (reverse renames — structure only, no data recovery needed)
- [ ] Run migration
- [ ] Verify tables renamed: `\dt` should show proposal, proposal_actions, etc.
- [ ] Commit: `feat(phase-5): migration rename comment → proposal tables/columns/indexes`

---

### Task 2: Regenerate schema.rs + Update replaceable SQL schema + Rust enum rename

**Files:**
- Modify: `crates/db/src/schema.rs` (regenerate via diesel print-schema)
- Modify: `crates/db/replaceable_schema/triggers.sql` (rename comment → proposal throughout)
- Modify: `crates/db/replaceable_schema/utils.sql` (rename comment → proposal throughout)
- Modify: `crates/db/src/enums.rs` (`CommentSortType` → `ProposalSortType`)

**What to do:**

1. **Regenerate schema.rs:**
```bash
export DATABASE_URL="postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1"
diesel print-schema > crates/db/src/schema.rs
```
If diesel print-schema uses the patch file and fails, edit schema.rs manually — remove all old `comment*` table definitions and add `proposal*` ones.

2. **triggers.sql** — global rename:
```bash
# Preview what needs changing:
grep -n "comment" crates/db/replaceable_schema/triggers.sql | grep -v "^--" | head -30
```
Rename throughout (case-sensitive):
- `'comment'` (table name string) → `'proposal'`
- `comment_count` → `proposal_count`
- `(comment).` → `(proposal).`
- `comment_id` → `proposal_id`
- `parent_comment_ids` → `parent_proposal_ids`
- `comment.path` → `proposal.path`
- `comment c` (alias) → `proposal p` (or keep alias `c` — just rename the table name)
- `comment_actions` → `proposal_actions`
- Function `r.post_or_comment` — rename to `r.post_or_proposal`
- `CALL r.post_or_comment ('comment')` → `CALL r.post_or_proposal ('proposal')`
- `CALL r.post_or_comment ('post')` → `CALL r.post_or_proposal ('post')`

3. **utils.sql** — same pattern, check for comment refs:
```bash
grep -n "comment" crates/db/replaceable_schema/utils.sql | grep -v "^--"
```

4. **enums.rs:**
```bash
grep -n "CommentSortType\|comment_sort_type" crates/db/src/enums.rs | head -10
```
Rename:
- `CommentSortType` enum → `ProposalSortType`
- `ExistingTypePath = "crate::schema::sql_types::CommentSortTypeEnum"` → `ProposalSortTypeEnum`

**Compile check after this task:**
```bash
cargo check --workspace 2>&1 | grep "^error" | head -20
```
Expected: many errors about `comment`-named types not found (fixed in Task 3). Zero unexpected errors about non-comment types.

- [ ] Regenerate/update schema.rs
- [ ] Update triggers.sql (all `comment` → `proposal`)
- [ ] Update utils.sql
- [ ] Rename `CommentSortType` → `ProposalSortType` in enums.rs
- [ ] `cargo check --workspace` — document remaining errors (expected: comment type not found)
- [ ] `cargo +nightly fmt`
- [ ] Commit: `refactor(phase-5): regenerate schema, update replaceable SQL, rename CommentSortType enum`

---

### Task 3: Mass Rust rename in db/ and db_views/ crates

**Files:**
- Modify: `crates/db/src/source/comment.rs` → rename types within (do NOT rename the file yet if it breaks things)
- Modify: `crates/db/src/source/comment_reply.rs`, `comment_report.rs`, etc.
- Modify: `crates/db/src/impls/comment.rs`, `comment_report.rs`, `comment_reply.rs`
- Modify: `crates/db_views/comment/` entire crate (rename crate: `app_108jobs_db_views_comment` → `app_108jobs_db_views_proposal`)
- Modify: all crates that depend on `app_108jobs_db_views_comment`

**Strategy — batch rename, then fix compilation errors:**

```bash
# Step 1: rename source files
cd crates/db/src/source/
for f in comment*.rs; do mv "$f" "${f/comment/proposal}"; done

cd crates/db/src/impls/
for f in comment*.rs; do mv "$f" "${f/comment/proposal}"; done

# Step 2: rename crate directory
mv crates/db_views/comment crates/db_views/proposal

# Step 3: global text rename in all Rust files
find crates/ -name "*.rs" -not -path "*/target/*" | xargs sed -i '' \
  -e 's/CommentSortType/ProposalSortType/g' \
  -e 's/comment_sort_type/proposal_sort_type/g' \
  -e 's/CommentReport/ProposalReport/g' \
  -e 's/comment_report/proposal_report/g' \
  -e 's/CommentReply/ProposalReply/g' \
  -e 's/comment_reply/proposal_reply/g' \
  -e 's/CommentActions/ProposalActions/g' \
  -e 's/comment_actions/proposal_actions/g' \
  -e 's/AdminPurgeComment/AdminPurgeProposal/g' \
  -e 's/admin_purge_comment/admin_purge_proposal/g' \
  -e 's/ModRemoveComment/ModRemoveProposal/g' \
  -e 's/mod_remove_comment/mod_remove_proposal/g' \
  -e 's/PersonCommentMention/PersonProposalMention/g' \
  -e 's/person_comment_mention/person_proposal_mention/g' \
  -e 's/CommentId\b/ProposalId/g' \
  -e 's/comment_id\b/proposal_id/g' \
  -e 's/\bComment\b/Proposal/g' \
  -e 's/\bcomment\b/proposal/g'
```

**WARNING:** The above sed is aggressive. Apply it and then fix false positives in compile errors. Do NOT rename:
- `delivery_rider_rating.comment` (free-text field, not the concept)
- `user_review.comment` (free-text field)
- `// comment` (code comments — be careful)
- The `crates/proposals/` crate (already named proposals, not comment)

**Then update Cargo.toml files:**
- `crates/db_views/proposal/Cargo.toml`: rename package from `app_108jobs_db_views_comment` to `app_108jobs_db_views_proposal`
- Root `Cargo.toml` workspace.dependencies: rename `app_108jobs_db_views_comment` → `app_108jobs_db_views_proposal`
- All crates with `app_108jobs_db_views_comment` dep: update to `app_108jobs_db_views_proposal`

**Update db/src/source/mod.rs, db/src/impls/mod.rs** to use renamed module names.

**Compile check:**
```bash
cargo check --workspace 2>&1 | grep "^error" | head -20
```
Fix all errors. Expected: mostly missing module names from renamed files, type mismatches from missed replacements.

- [ ] Rename source files in `db/src/source/` and `db/src/impls/`
- [ ] Rename `crates/db_views/comment/` → `crates/db_views/proposal/`
- [ ] Run global sed rename on all .rs files
- [ ] Update Cargo.toml files (package name + deps)
- [ ] Update mod.rs files to reference renamed modules
- [ ] `cargo check --workspace` → 0 errors
- [ ] `cargo nextest run -p app_108jobs_contract_tests` → 16/16
- [ ] `cargo +nightly fmt`
- [ ] Commit: `refactor(phase-5): rename Comment→Proposal types in db and db_views crates`

---

### Task 4: Mass Rust rename in api/handler crates + route paths + final gates

**Files:**
- Modify: `crates/api/api_common/src/comment.rs` → rename file and types within
- Modify: all files in `crates/proposals/src/` (rename functions like `create_comment` → `create_proposal`)
- Modify: `src/api_routes.rs` (route paths `/comment/*` → `/proposal/*`)
- Modify: any remaining api crates with `Comment` type references

**What to do:**

1. Rename `crates/api/api_common/src/comment.rs` → `proposal.rs` (or rename types within if the module is re-exported)

2. In `crates/proposals/src/handlers/`, rename all handler functions:
   - `create_comment` → `create_proposal`
   - `delete_comment` → `delete_proposal`
   - `edit_comment` → `edit_proposal`
   - etc.

3. In `src/api_routes.rs`, update route paths:
```bash
grep -n "comment\|Comment" src/api_routes.rs | head -20
```
Change `/comment` path segments to `/proposal`.

4. Run any remaining global sed:
```bash
find crates/ src/ -name "*.rs" -not -path "*/target/*" | xargs grep -l "\bcomment\b\|Comment" | head -20
```

5. Final compile + test gate:
```bash
cargo check --workspace 2>&1 | grep "^error" | head -10
cargo nextest run -p app_108jobs_contract_tests 2>&1 | tail -3
cargo +nightly fmt
```

- [ ] Rename `api_common/src/comment.rs` → `proposal.rs` and update types
- [ ] Update function names in `crates/proposals/src/`
- [ ] Update route paths in `src/api_routes.rs`
- [ ] Fix remaining compile errors
- [ ] `cargo check --workspace` → 0 errors
- [ ] `cargo nextest run -p app_108jobs_contract_tests` → 16/16
- [ ] `cargo +nightly fmt`
- [ ] Commit: `refactor(phase-5): rename comment→proposal in api crates and route paths`
