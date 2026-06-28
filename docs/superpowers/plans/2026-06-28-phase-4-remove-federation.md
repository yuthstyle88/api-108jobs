# Phase 4 — Remove ActivityPub / Federation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Goal:** Completely remove all ActivityPub / federation code and DB infrastructure from api-108jobs. The instance is confirmed non-federating.

**Architecture:** Three-pass removal — (1) Rust code that calls federation methods, (2) federation methods and traits themselves, (3) DB migration dropping federation-only tables/columns and regenerate schema.rs. Triggers.sql updated inline with the Rust pass.

**Tech Stack:** Rust/Actix/Diesel, PostgreSQL, Cargo workspace (~41 crates)

## Global Constraints

- `cargo check --workspace` must remain at 0 errors after every task.
- `cargo nextest run -p app_108jobs_contract_tests` must stay at 16/16.
- `cargo +nightly fmt` before every commit.
- Branch: `phase-4/remove-federation`. Never touch main directly.
- **KEEP `person.private_key` and `person.shared_key`** — these are 108Jobs OTP/KYC keys (added by migration `2025-08-04-014547_add_otp_fields_to_local_user`), NOT federation signing keys.
- **KEEP `person.contacts` and `person.skills` JSONB** — 108Jobs business data.
- After Task 3 (DB migration), run `cargo run -p diesel_cli -- print-schema` to regenerate `crates/db/src/schema.rs`. Or rely on `REQUIRED DIESEL DERIVE` flags in each crate.
- Never drop production data without a documented migration.
- Stop-and-ask triggers: any column not clearly federation-only, any table with 108Jobs data.

---

## Columns/Tables to Drop (Migration)

**Tables to DROP entirely:**
- `sent_activity` — pure federation (tracks outgoing AP activities)
- `received_activity` — pure federation (deduplication of incoming AP activities)

**Columns to DROP from existing tables:**

| Table | Columns to drop |
|---|---|
| `person` | `ap_id`, `local`, `inbox_url` |
| `post` | `ap_id`, `local` |
| `comment` | `ap_id`, `local` |
| `category` | `ap_id`, `local`, `followers_url`, `inbox_url`, `moderators_url`, `featured_url` |
| `site` | `ap_id`, `inbox_url`, `private_key`, `public_key` |

**SQL types to DROP:**
- `actor_type_enum`

**DO NOT drop:**
- `person.private_key` — 108Jobs OTP/KYC key
- `person.shared_key` — 108Jobs OTP/KYC key
- Any 108Jobs business table or column

---

## Rust Symbols to Remove

**Traits (crates/db/src/traits.rs):**
- `ApubActor` trait entirely
- `Reportable::resolve_apub()` method (the rest of Reportable may stay if used)

**Utility functions:**
- `generate_inbox_url()` in `crates/api/api_utils/src/utils.rs`
- `placeholder_apub_url()` in `crates/db/src/source/mod.rs`
- `Comment::local_url()` in `crates/db/src/impls/comment.rs`
- `Post::local_url()` in `crates/db/src/impls/post.rs`

**Impl methods:**
- `impl ApubActor for Person` in `crates/db/src/impls/person.rs`
- `Person::upsert()` on_conflict(`person::ap_id`) — replace with plain `INSERT`
- `Post::insert_apub()` in `crates/db/src/impls/post.rs`
- `Post::read_from_apub_id()` in `crates/db/src/impls/post.rs`
- `Comment::read_from_apub_id()` in `crates/db/src/impls/comment.rs`
- `impl ApubActor for Category` in `crates/db/src/impls/category.rs`
- `Category::insert_apub()` in `crates/db/src/impls/category.rs`
- `Site::read_from_apub_id()` in `crates/db/src/impls/site.rs`
- `PostReport::resolve_apub()` in `crates/db/src/impls/post_report.rs`
- `CommentReport::resolve_apub()` in `crates/db/src/impls/comment_report.rs`
- `CategoryReport::resolve_apub()` in `crates/db/src/impls/category_report.rs`

**Struct fields to remove** (from source types AND insert/update forms):
- `person.rs`: `ap_id`, `local`, `inbox_url` (keep `private_key`, `shared_key`)
- `post.rs`: `ap_id`, `local`
- `comment.rs`: `ap_id`, `local`
- `category.rs`: `ap_id`, `local`, `followers_url`, `inbox_url`, `moderators_url`, `featured_url`
- `site.rs`: `ap_id`, `inbox_url`, `private_key`, `public_key`

**Aliased fields in crates/db/src/lib.rs (lines ~206-248):**
- Remove all `person::ap_id`, `person::inbox_url`, `person::private_key` (federation), `person::shared_key` (federation) aliases
- Keep `person::private_key` and `person::shared_key` if they appear in 108Jobs-specific alias groups

**Enum:**
- `ActorTypeEnum` in `crates/db/src/enums.rs` — remove entirely

---

### Task 1: Remove call sites that USE federation methods

**Files:**
- Modify: `crates/api/api_crud/src/user/create.rs`
- Modify: `crates/api/api_crud/src/site/create.rs`
- Modify: `crates/routes/src/utils/setup_local_site.rs`
- Modify: `crates/api/api_utils/src/build_response.rs`
- Modify: `crates/routes/src/feeds.rs`
- Modify: `crates/db/replaceable_schema/triggers.sql`
- Modify: `crates/db/src/test_data.rs`
- Modify: `crates/db/src/lib.rs` (aliased federation fields)

**What to do:**

1. **`user/create.rs`**: Remove `generate_inbox_url` import and call. Remove `inbox_url: Some(generate_inbox_url()?)` from `PersonInsertForm`. Do NOT supply a value for `inbox_url` — the field will be removed from the struct in Task 2.

2. **`site/create.rs`**: Remove `generate_inbox_url` import and call. Remove `inbox_url` from `SiteInsertForm`.

3. **`setup_local_site.rs`**: Remove all `inbox_url` field assignments.

4. **`build_response.rs`**: Remove `comment.local_url(...)` and `post.local_url(...)` calls. These are used for `ap_id`-related response fields — remove the entire field from the response, or set it to a static empty string if the field is required by a client. Check what field name this populates. If `ap_id` is in the API response type, it must be removed there too — trace to `crates/api/api_common/src/` response structs.

5. **`feeds.rs`**: Remove all `local_url()` calls and any code that sets `ap_id` in RSS/Atom feed items. Federation URLs are not needed in feeds for a non-federating instance.

6. **`triggers.sql`**: Remove the `comment_change_values()` and `post_change_values()` PL/pgSQL trigger functions and their corresponding `CREATE TRIGGER` statements. These set `ap_id` on insert/update. Also remove `r.local_url()` function if it exists. The `ap_id` column will be dropped in Task 3.

7. **`test_data.rs`**: Remove all federation field usages (`ap_id`, `inbox_url`, `local`, etc.) from test data generation.

8. **`lib.rs` (DB crate)**: Remove aliased fields for `person::ap_id`, `person::inbox_url`. Review whether any federation-specific join alias groups can be deleted entirely.

**Compile gate:** `cargo check --workspace` → 0 errors (will fail until Task 2 removes the struct fields from the Diesel model).

- [ ] Read each file before editing
- [ ] Edit `user/create.rs` — remove `generate_inbox_url` import and usage
- [ ] Edit `site/create.rs` — remove `generate_inbox_url` import and usage
- [ ] Edit `setup_local_site.rs` — remove `inbox_url` field assignments
- [ ] Edit `build_response.rs` — remove `local_url()` calls, trace and remove `ap_id` from response structs in `api_common`
- [ ] Edit `feeds.rs` — remove `local_url()` calls
- [ ] Edit `triggers.sql` — remove federation trigger functions and CREATE TRIGGER statements
- [ ] Edit `test_data.rs` — remove federation fields from test data
- [ ] Edit `lib.rs` — remove aliased federation fields
- [ ] `cargo check --workspace` → 0 errors (may have residual errors from struct fields not yet removed — document them, that is OK for this task)
- [ ] `cargo nextest run -p app_108jobs_contract_tests` → 16/16
- [ ] `cargo +nightly fmt`
- [ ] Commit: `refactor(phase-4): remove federation call sites and triggers`

---

### Task 2: Remove federation struct fields, methods, and traits

**Files:**
- Modify: `crates/db/src/source/person.rs`
- Modify: `crates/db/src/source/post.rs`
- Modify: `crates/db/src/source/comment.rs`
- Modify: `crates/db/src/source/category.rs`
- Modify: `crates/db/src/source/site.rs`
- Modify: `crates/db/src/source/mod.rs`
- Modify: `crates/db/src/traits.rs`
- Modify: `crates/db/src/enums.rs`
- Modify: `crates/db/src/impls/person.rs`
- Modify: `crates/db/src/impls/post.rs`
- Modify: `crates/db/src/impls/comment.rs`
- Modify: `crates/db/src/impls/category.rs`
- Modify: `crates/db/src/impls/site.rs`
- Modify: `crates/db/src/impls/post_report.rs`
- Modify: `crates/db/src/impls/comment_report.rs`
- Modify: `crates/db/src/impls/category_report.rs`
- Modify: `crates/api/api_utils/src/utils.rs`

**What to do:**

1. **Source types** — remove fields from every struct listed in "Struct fields to remove" above. For each file, remove from: the main struct, `*InsertForm`, `*UpdateForm`. Remember: `person.private_key` and `person.shared_key` are 108Jobs OTP/KYC — KEEP them.

2. **`mod.rs`** — delete `placeholder_apub_url()` function.

3. **`traits.rs`** — delete `ApubActor` trait entirely. Delete `Reportable::resolve_apub()` method. If `Reportable` becomes empty, delete the whole trait.

4. **`enums.rs`** — delete `ActorTypeEnum`. Remove its `#[diesel(sql_type = ...)]` derive and the DB-mapped type.

5. **`impls/person.rs`** — delete `impl ApubActor for Person`. Remove `on_conflict(person::ap_id)` from `Person::upsert()` — replace with a plain insert or remove the upsert if unused.

6. **`impls/post.rs`** — delete `Post::insert_apub()`, `Post::read_from_apub_id()`, `Post::local_url()`.

7. **`impls/comment.rs`** — delete `Comment::read_from_apub_id()`, `Comment::local_url()`.

8. **`impls/category.rs`** — delete `impl ApubActor for Category`, `Category::insert_apub()`. Remove all federation field assignments in `Category::create()`.

9. **`impls/site.rs`** — delete `Site::read_from_apub_id()`.

10. **`impls/post_report.rs`**, **`impls/comment_report.rs`**, **`impls/category_report.rs`** — delete `resolve_apub()` implementations.

11. **`utils.rs`** — delete `generate_inbox_url()`.

**Compile gate:** `cargo check --workspace` → 0 errors (will fail until Task 3 regenerates schema.rs; document and skip for now if schema.rs still has old columns).

- [ ] Read all files before editing (can batch-read)
- [ ] Remove struct fields from all 5 source files (keep `person.private_key`, `person.shared_key`)
- [ ] Remove `placeholder_apub_url()` from `mod.rs`
- [ ] Remove `ApubActor` and `Reportable::resolve_apub` from `traits.rs`
- [ ] Remove `ActorTypeEnum` from `enums.rs`
- [ ] Remove apub impl blocks from `impls/person.rs`
- [ ] Remove apub methods from `impls/post.rs`
- [ ] Remove apub methods from `impls/comment.rs`
- [ ] Remove apub impl + fields from `impls/category.rs`
- [ ] Remove apub method from `impls/site.rs`
- [ ] Remove `resolve_apub` from all 3 report impl files
- [ ] Remove `generate_inbox_url()` from `utils.rs`
- [ ] `cargo check --workspace` → 0 errors
- [ ] `cargo nextest run -p app_108jobs_contract_tests` → 16/16
- [ ] `cargo +nightly fmt`
- [ ] Commit: `refactor(phase-4): remove federation structs, traits, and methods`

---

### Task 3: DB migration — drop federation tables/columns + regenerate schema.rs

**Files:**
- Create: `migrations/2026-06-28-HHMMSS-0000_remove_federation/up.sql`
- Create: `migrations/2026-06-28-HHMMSS-0000_remove_federation/down.sql`
- Modify: `crates/db/src/schema.rs` (regenerated via diesel print-schema OR manual removal)

**Migration order (FK-safe):**

```sql
-- up.sql

-- 1. Drop pure federation tables first (no FK dependents in 108Jobs code)
DROP TABLE IF EXISTS sent_activity;
DROP TABLE IF EXISTS received_activity;

-- 2. Drop federation columns from each table (check FK constraints first)
-- person
ALTER TABLE person DROP COLUMN IF EXISTS ap_id;
ALTER TABLE person DROP COLUMN IF EXISTS local;
ALTER TABLE person DROP COLUMN IF EXISTS inbox_url;
-- post
ALTER TABLE post DROP COLUMN IF EXISTS ap_id;
ALTER TABLE post DROP COLUMN IF EXISTS local;
-- comment
ALTER TABLE comment DROP COLUMN IF EXISTS ap_id;
ALTER TABLE comment DROP COLUMN IF EXISTS local;
-- category
ALTER TABLE category DROP COLUMN IF EXISTS ap_id;
ALTER TABLE category DROP COLUMN IF EXISTS local;
ALTER TABLE category DROP COLUMN IF EXISTS followers_url;
ALTER TABLE category DROP COLUMN IF EXISTS inbox_url;
ALTER TABLE category DROP COLUMN IF EXISTS moderators_url;
ALTER TABLE category DROP COLUMN IF EXISTS featured_url;
-- site
ALTER TABLE site DROP COLUMN IF EXISTS ap_id;
ALTER TABLE site DROP COLUMN IF EXISTS inbox_url;
ALTER TABLE site DROP COLUMN IF EXISTS private_key;
ALTER TABLE site DROP COLUMN IF EXISTS public_key;

-- 3. Drop actor_type_enum (used only by sent_activity which is now dropped)
DROP TYPE IF EXISTS actor_type_enum;
```

```sql
-- down.sql (rollback: re-add columns with NULL defaults, tables empty)
-- NOTE: Data in these columns is LOST on up.sql. This rollback only restores structure.
CREATE TABLE IF NOT EXISTS received_activity (
  ap_id text PRIMARY KEY,
  published timestamptz NOT NULL DEFAULT now()
);
-- ... (add other tables/columns as needed for rollback)
-- For simplicity, down.sql can be a documented no-op if data loss is accepted.
```

**After migration, regenerate schema.rs:**

Option A (if diesel CLI is available):
```bash
diesel print-schema --database-url=$DATABASE_URL > crates/db/src/schema.rs
```

Option B (manual): Remove the dropped columns/tables from `crates/db/src/schema.rs` by editing it directly. This is acceptable since schema.rs is auto-generated and the DB is authoritative.

**Before running the migration:**
- Verify no remaining Rust code references the columns being dropped (Task 1+2 must be complete).
- Verify `cargo check --workspace` → 0 errors (with old schema.rs having the columns).

**Compile gate after regenerating schema.rs:** `cargo check --workspace` → 0 errors.

- [ ] Create migration directory with timestamp
- [ ] Write `up.sql` (drop tables → drop columns → drop enum type)
- [ ] Write `down.sql` (structure-only rollback with comment noting data loss)
- [ ] Run migration against local DB: `diesel migration run`
- [ ] Regenerate `crates/db/src/schema.rs` (via diesel CLI or manual edit)
- [ ] `cargo check --workspace` → 0 errors
- [ ] `cargo nextest run -p app_108jobs_contract_tests` → 16/16
- [ ] `cargo +nightly fmt`
- [ ] Commit: `feat(phase-4): migration to drop federation tables/columns; regenerate schema.rs`
