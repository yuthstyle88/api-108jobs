# Phase 3 — Carve 2: `db` Crate

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Merge `crates/db_schema/` + `crates/db_schema_file/` + `crates/db_schema_setup/` into a single `crates/db/` crate named `app_108jobs_db`. Eliminates the Lemmy-style `_schema_file`/`_schema_setup` naming and unifies the database layer into one bounded context.

**Architecture:** Physical file merge: db_schema_file's `schema.rs`/`enums.rs` become internal modules of `db`; db_schema_setup's migration runner becomes `db::migrations`; db_schema's source/impls/traits become `db` top-level modules. All 35–38 external consumers are updated via mass sed. Internal cross-crate references are fixed to use `crate::` paths. The three old crates are deleted.

**Tech Stack:** Rust, Cargo workspace, `sed`, `cargo check`, `cargo nextest`

## Global Constraints

- **No behavior change.** The public API surface of the merged `app_108jobs_db` is the union of the 3 crates' public APIs. No types, functions, or module paths change externally — only crate-level prefixes change.
- **Module paths after merge:**
  - `app_108jobs_db_schema::source::post::Post` → `app_108jobs_db::source::post::Post`
  - `app_108jobs_db_schema_file::schema::post` → `app_108jobs_db::schema::post`
  - `app_108jobs_db_schema_file::enums::PostStatus` → `app_108jobs_db::enums::PostStatus`
  - `app_108jobs_db_schema_setup::run(...)` → `app_108jobs_db::migrations::run(...)`
  - `app_108jobs_db_schema_setup::Options` → `app_108jobs_db::migrations::Options`
  - `app_108jobs_db_schema::traits::*` → `app_108jobs_db::traits::*` (unchanged)
  - `app_108jobs_db_schema::newtypes::*` → `app_108jobs_db::newtypes::*`
- **`"full"` feature preserved.** The merged crate must have the `"full"` feature that gates Diesel derives. All consumers using `app_108jobs_db_schema = { features = ["full"] }` migrate to `app_108jobs_db = { features = ["full"] }`.
- **`diesel_ltree.patch`** in `crates/db_schema_file/` must be preserved at `crates/db/` (referenced by `diesel.toml` at line 3).
- **`migrations/` directory** (SQL files) lives at `crates/db/migrations/` after the move.
- **`replaceable_schema/`** lives at `crates/db/replaceable_schema/` — update `REPLACEABLE_SCHEMA_PATH` string in the migration runner.
- **Gate:** `cargo check --workspace` exits 0 (zero errors, zero warnings). `cargo nextest run -p app_108jobs_contract_tests` 16/16 pass.
- **Nightly fmt** after all changes.
- Commit: `refactor(phase-3): merge db_schema + db_schema_file + db_schema_setup → app_108jobs_db`.

## Context

Current state:
- `crates/db_schema/` — 22K LOC, crate `app_108jobs_db_schema`, 35 external consumers; contains source models, impls, traits, newtypes, aliases; depends on `app_108jobs_db_schema_file`
- `crates/db_schema_file/` — 2.4K LOC, crate `app_108jobs_db_schema_file`, 38 external consumers; contains only `schema.rs` (Diesel table macros) + `enums.rs` (DB enum types); `diesel_ltree.patch` at crate root
- `crates/db_schema_setup/` — 541 LOC, crate `app_108jobs_db_schema_setup`, 1 external consumer (main binary `src/lib.rs`); contains migration runner `run()`, `Options`, `Branch`; SQL files in `migrations/` and `replaceable_schema/`

---

### Task 1: Physical file merge — create `crates/db/`

**Files:**
- Create: `crates/db/` directory tree
- Move: `crates/db_schema/src/` → `crates/db/src/` (keep all sub-structure)
- Move: `crates/db_schema_file/src/schema.rs` → `crates/db/src/schema.rs`
- Move: `crates/db_schema_file/src/enums.rs` → `crates/db/src/enums.rs`
- Move: `crates/db_schema_setup/src/lib.rs` content → `crates/db/src/migrations.rs`
- Move: `crates/db_schema_setup/src/diff_check.rs` → `crates/db/src/diff_check.rs`
- Move: `crates/db_schema_setup/migrations/` → `crates/db/migrations/`
- Move: `crates/db_schema_setup/replaceable_schema/` → `crates/db/replaceable_schema/`
- Move: `crates/db_schema_file/diesel_ltree.patch` → `crates/db/diesel_ltree.patch`
- Create: `crates/db/Cargo.toml` (merged from all 3)
- Write: `crates/db/src/lib.rs` (merged lib.rs)

**Interfaces:**
- Produces: `crates/db/` with all content; `app_108jobs_db` crate compiles in isolation (before updating consumers)

- [ ] **Step 1: Copy source files from db_schema**

```bash
cp -r crates/db_schema/src/ crates/db/src/
```

- [ ] **Step 2: Copy schema.rs and enums.rs from db_schema_file**

```bash
cp crates/db_schema_file/src/schema.rs crates/db/src/schema.rs
cp crates/db_schema_file/src/enums.rs crates/db/src/enums.rs
```

- [ ] **Step 3: Create `crates/db/src/migrations.rs` from db_schema_setup**

Copy `crates/db_schema_setup/src/lib.rs` to `crates/db/src/migrations.rs`:
```bash
cp crates/db_schema_setup/src/lib.rs crates/db/src/migrations.rs
```

Then in `crates/db/src/migrations.rs`:
1. Change `mod diff_check;` to reference the sibling module: this is a private module, so in `migrations.rs` the declaration stays as `mod diff_check;` but the file must be at `crates/db/src/diff_check.rs` (next step handles this)
2. Update `REPLACEABLE_SCHEMA_PATH`:
   ```rust
   // OLD:
   const REPLACEABLE_SCHEMA_PATH: &str = "crates/db_schema_setup/replaceable_schema";
   // NEW:
   const REPLACEABLE_SCHEMA_PATH: &str = "crates/db/replaceable_schema";
   ```

- [ ] **Step 4: Move diff_check.rs**

```bash
cp crates/db_schema_setup/src/diff_check.rs crates/db/src/diff_check.rs
```

Note: `diff_check.rs` is declared as `mod diff_check;` inside `migrations.rs`. When `migrations.rs` is a module file (not `lib.rs`), Rust looks for `diff_check.rs` at `src/migrations/diff_check.rs` OR uses `#[path]` annotation. To keep it simple, use a path attribute in `migrations.rs`:

```rust
// In migrations.rs, change:
mod diff_check;
// to:
#[path = "../diff_check.rs"]
mod diff_check;
```

OR alternatively, create `crates/db/src/migrations/` directory and put files there. The simpler approach: put `diff_check.rs` at `crates/db/src/diff_check.rs` and use `#[path]`:

```rust
#[cfg(test)]
#[path = "diff_check.rs"]
mod diff_check;
```

- [ ] **Step 5: Move SQL directories**

```bash
cp -r crates/db_schema_setup/migrations/ crates/db/migrations/
cp -r crates/db_schema_setup/replaceable_schema/ crates/db/replaceable_schema/
```

- [ ] **Step 6: Move `diesel_ltree.patch`**

```bash
cp crates/db_schema_file/diesel_ltree.patch crates/db/diesel_ltree.patch
```

Check `diesel.toml` for the reference:
```bash
grep -n "diesel_ltree.patch\|patch_file" diesel.toml
```

Update if needed to reference `crates/db/diesel_ltree.patch`.

- [ ] **Step 7: Write merged `crates/db/src/lib.rs`**

The merged lib.rs combines the old `db_schema/src/lib.rs` with additions for the new modules. Replace `crates/db/src/lib.rs` (which was copied from `db_schema/src/lib.rs`) with:

```rust
#[cfg(feature = "full")]
#[macro_use]
extern crate diesel;
#[cfg(feature = "full")]
#[macro_use]
extern crate diesel_derive_newtype;

// From db_schema_file (now internal):
pub mod enums;
#[cfg(feature = "full")]
pub mod schema;

// Migration runner (from db_schema_setup):
pub mod migrations;

#[cfg(feature = "full")]
pub mod impls;
pub mod newtypes;
pub mod sensitive;
#[cfg(feature = "full")]
pub mod aliases {
  use crate::schema::{
    category_actions,
    instance_actions,
    local_user,
    person,
  };
  diesel::alias!(
    category_actions as creator_category_actions: CreatorcategoryActions,
    instance_actions as creator_home_instance_actions: CreatorHomeInstanceActions,
    instance_actions as creator_category_instance_actions: CreatorcategoryInstanceActions,
    instance_actions as creator_local_instance_actions: CreatorLocalInstanceActions,
    local_user as creator_local_user: CreatorLocalUser,
    person as person1: Person1,
    person as person2: Person2,
  );
}
pub mod source;
#[cfg(all(feature = "full", any(test, feature = "test-utils")))]
pub mod test_data;
#[cfg(feature = "full")]
pub mod traits;
#[cfg(feature = "full")]
pub mod utils;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
#[cfg(feature = "full")]
use {
  crate::schema::{category_actions, instance_actions, person},
  diesel::query_source::AliasedField,
};

// [all the enum definitions and type aliases from the old db_schema/src/lib.rs go here]
// SearchSortType, CategorySortType, SearchType, ModlogActionType, InboxDataType,
// PersonContentType, ReportType, PostFeatureType, LikeType, assert_length! macro,
// Person1AliasAllColumnsTuple, Person2AliasAllColumnsTuple, etc.
```

The key changes from the old `db_schema/src/lib.rs`:
1. Add `pub mod enums;`, `pub mod schema;`, `pub mod migrations;` at the top
2. In `pub mod aliases`, change `use app_108jobs_db_schema_file::schema::*` → `use crate::schema::*`
3. In the `#[cfg(feature = "full")] use { ... }` block at the bottom, change `app_108jobs_db_schema_file::schema::*` → `crate::schema::*`

- [ ] **Step 8: Fix internal references within `crates/db/src/`**

Run:
```bash
grep -rn "app_108jobs_db_schema_file\|app_108jobs_db_schema_setup" crates/db/src/ --include="*.rs" | grep -v "target/"
```

For each match, replace with `crate::` equivalent:
- `use app_108jobs_db_schema_file::schema::` → `use crate::schema::`
- `use app_108jobs_db_schema_file::enums::` → `use crate::enums::`
- `app_108jobs_db_schema_file::schema::` (unqualified) → `crate::schema::`

Also: the `db_schema/src/` files that import from `app_108jobs_db_schema_file` internally — update those:
```bash
find crates/db/src -name "*.rs" -exec sed -i '' 's/app_108jobs_db_schema_file::/crate::/g' {} +
```

- [ ] **Step 9: Create merged `crates/db/Cargo.toml`**

Merge the dependencies from all 3 crates. Start from `crates/db_schema/Cargo.toml` as the base, then add any deps unique to `db_schema_file` and `db_schema_setup`.

Key points:
- `name = "app_108jobs_db"` (NOT `app_108jobs_db_schema`)
- Remove the internal cross-dep: `app_108jobs_db_schema_setup = { workspace = true, optional = true }` (now merged in)
- Remove: `app_108jobs_db_schema_file = { workspace = true }` (now merged in)
- Keep the `[features]` section from db_schema: `full = [...]`, `test-utils = [...]`, etc.
- Add any deps from db_schema_setup that aren't already in db_schema: `diesel_migrations`, `chrono`, `anyhow`

```toml
[package]
name = "app_108jobs_db"
# ... rest of package metadata

[features]
full = [
  # ... same as db_schema's full feature, but remove app_108jobs_db_schema_file and app_108jobs_db_schema_setup
  "diesel/postgres",
  # etc.
]
test-utils = ["full"]

[dependencies]
app_108jobs_core = { workspace = true }
# ... all deps from db_schema Cargo.toml
diesel_migrations = { workspace = true }
chrono = { workspace = true }
anyhow = { workspace = true }
# ... any additional deps from db_schema_setup not already in db_schema
```

- [ ] **Step 10: Create `crates/db/` directory structure**

Ensure the directory has the right structure:
```bash
ls crates/db/
# Expected: Cargo.toml, src/, migrations/, replaceable_schema/, diesel_ltree.patch
ls crates/db/src/
# Expected: lib.rs, schema.rs, enums.rs, migrations.rs, diff_check.rs, source/, impls/, traits.rs, newtypes.rs, sensitive.rs, utils.rs, utils/, test_data.rs
```

- [ ] **Step 11: Attempt to compile `app_108jobs_db` in isolation**

```bash
cargo check -p app_108jobs_db --all-features 2>&1 | grep "^error" | head -20
```

Fix errors one by one using the compiler output. Common issues:
- Missing deps in Cargo.toml
- Module path errors (missed `crate::` replacement)
- `diff_check` module path issue in `migrations.rs`
- Feature flag issues

- [ ] **Step 12: Commit the new crate (before touching consumers)**

```bash
git add crates/db/
git commit -m "refactor(phase-3): create crates/db/ from merged db_schema + db_schema_file + db_schema_setup"
```

---

### Task 2: Update consumers + delete old crates

**Files:**
- Modify: All 35-38 `*.toml` files that dep on the old 3 crates
- Modify: All `*.rs` files with `app_108jobs_db_schema*` imports
- Modify: `src/lib.rs` — change `app_108jobs_db_schema_setup::run(...)` → `app_108jobs_db::migrations::run(...)`
- Modify: root `Cargo.toml` — update workspace members + workspace.dependencies
- Modify: `diesel.toml` — update patch_file path to `crates/db/diesel_ltree.patch`
- Delete: `crates/db_schema/`, `crates/db_schema_file/`, `crates/db_schema_setup/`

**Interfaces:**
- Produces: All consumers import from `app_108jobs_db`; old 3 crates gone; workspace compiles clean

**Strategy:** Mass sed replacements (longest-match first to avoid substring corruption), then compile-driven cleanup, then delete old crates.

- [ ] **Step 1: Add `app_108jobs_db` to workspace root `Cargo.toml`**

In `[workspace.dependencies]`, add:
```toml
app_108jobs_db = { version = "=1.0.0-alpha.5", path = "./crates/db" }
```

In `[workspace] members`, add `"crates/db"`.

- [ ] **Step 2: Mass-replace crate identifiers in `.toml` files (longest first)**

```bash
# Replace _file variant first (longest, avoids substring match)
find . -name "*.toml" -not -path "*/target/*" \
  -exec sed -i '' 's/app_108jobs_db_schema_file/app_108jobs_db/g' {} +

# Then _setup variant
find . -name "*.toml" -not -path "*/target/*" \
  -exec sed -i '' 's/app_108jobs_db_schema_setup/app_108jobs_db/g' {} +

# Then _schema (now safe since _file and _setup already replaced)
find . -name "*.toml" -not -path "*/target/*" \
  -exec sed -i '' 's/app_108jobs_db_schema/app_108jobs_db/g' {} +
```

- [ ] **Step 3: Mass-replace crate identifiers in `.rs` files (longest first)**

```bash
find . -name "*.rs" -not -path "*/target/*" \
  -exec sed -i '' 's/app_108jobs_db_schema_file::/app_108jobs_db::/g' {} +
find . -name "*.rs" -not -path "*/target/*" \
  -exec sed -i '' 's/app_108jobs_db_schema_setup::/app_108jobs_db::migrations::/g' {} +
find . -name "*.rs" -not -path "*/target/*" \
  -exec sed -i '' 's/app_108jobs_db_schema::/app_108jobs_db::/g' {} +
```

- [ ] **Step 4: Fix the main binary migration call**

In `src/lib.rs` (lines ~124-133):
```rust
// OLD:
app_108jobs_db_schema_setup::Options::default().run()
app_108jobs_db_schema_setup::Options::default().revert()
app_108jobs_db_schema_setup::run(options, &SETTINGS.get_database_url())?;
// NEW (after sed, should already be):
app_108jobs_db::migrations::Options::default().run()
app_108jobs_db::migrations::Options::default().revert()
app_108jobs_db::migrations::run(options, &SETTINGS.get_database_url())?;
```

Verify the sed in Step 3 handled this correctly. If not, fix manually.

- [ ] **Step 5: Update `diesel.toml` patch_file path**

```bash
grep -n "diesel_ltree\|patch_file" diesel.toml
```

Change from `crates/db_schema_file/diesel_ltree.patch` to `crates/db/diesel_ltree.patch`.

- [ ] **Step 6: Remove old workspace entries from root `Cargo.toml`**

Remove from `members`:
```toml
    "crates/db_schema",
    "crates/db_schema_file",
    "crates/db_schema_setup",
```

Remove from `[workspace.dependencies]`:
```toml
app_108jobs_db_schema = { ... }
app_108jobs_db_schema_file = { ... }
app_108jobs_db_schema_setup = { ... }
```

(These should already be gone after the sed, but verify they're not duplicated or partially updated.)

- [ ] **Step 7: Full workspace compile check**

```bash
cargo check --workspace 2>&1 | grep "^error" | head -20
```

Fix errors using the compiler output. Common issues:
- Feature flags: `app_108jobs_db = { workspace = true, features = ["full"] }` — check that `features = ["full"]` references still work in the merged crate
- Any `app_108jobs_db_schema_file` or `app_108jobs_db_schema_setup` or `app_108jobs_db_schema` string still in code

Verification:
```bash
grep -rn "app_108jobs_db_schema" . --include="*.rs" --include="*.toml" | grep -v "target/" | head -5
```
Expected: zero matches (or only in plan/doc files).

- [ ] **Step 8: Delete the 3 old crates**

```bash
rm -rf crates/db_schema/ crates/db_schema_file/ crates/db_schema_setup/
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
git commit -m "refactor(phase-3): merge db_schema + db_schema_file + db_schema_setup → app_108jobs_db"
```

---

## Self-Review

**Spec coverage:**
- ✅ `crates/db_schema/` deleted → `crates/db/`
- ✅ `crates/db_schema_file/` deleted → content merged into `crates/db/src/`
- ✅ `crates/db_schema_setup/` deleted → content merged as `db::migrations`
- ✅ `app_108jobs_db_schema` → `app_108jobs_db` in all consumers
- ✅ `app_108jobs_db_schema_file` → `app_108jobs_db` in all consumers
- ✅ `app_108jobs_db_schema_setup` → `app_108jobs_db::migrations` in all consumers
- ✅ `"full"` feature preserved in merged crate
- ✅ `diesel_ltree.patch` preserved at `crates/db/`
- ✅ Migration SQL files at `crates/db/migrations/`
- ✅ `REPLACEABLE_SCHEMA_PATH` updated
- ✅ Gate: `cargo check --workspace` + 16/16 contract tests green

**No placeholders.** Compile-driven: any missed reference surfaces as a compiler error.
