# Fix flaky test suite — serialize nextest (no per-test DB isolation)

- Date: 2026-06-26
- Branch: `fix/test-migration-race`

## Problem

`cargo test` (the `cargo nextest run --profile ci` job) is red on `main`. Tests
fail in two ways, both at ~0.4s:

1. **Migration race** — wallet tests panic in DB setup with
   `duplicate key value violates unique constraint "pg_type_typname_nsp_index"`
   (two processes run the enum `CREATE TYPE` migrations at once).
2. **Data interference** — `actor_language` / `category` tests fail with
   `assertion failed: (left == right)` (tests clobber each other's rows).

## Root cause

Every test calls `build_db_pool_for_tests()` → `db_schema_setup::run()`, which
**migrates and seeds the one shared CI Postgres**. nextest runs every test in
its **own process across a parallel pool**, so many tests hit that shared
database simultaneously. There is **no per-test isolation** (no per-test DB,
schema, or transaction rollback). Concurrency therefore produces both the
migration race and the row-level interference. The CI job comment in
`.github/workflows/ci.yml` already acknowledges "test-isolation and assertion
failures".

A first attempt added an advisory-lock double-check inside `db_schema_setup::run`
— it did **not** fix it (CI still red, same error), and it couldn't address the
data-interference class at all. That approach was reverted.

## Fix

Run the suite **serially**: `test-threads = 1` in `[profile.ci]` of
`.config/nextest.toml`. One test runs at a time, so:
- the first test migrates; later tests find migrations already applied and skip
  via the existing lock-free fast path → no `CREATE TYPE` race;
- no two tests touch shared table data concurrently → no interference.

This is the correct fix for a suite without per-test DB isolation. Trade-off:
slower wall-time (the parallelism that the nextest config bought is given up
until proper per-test isolation exists).

## Scope (Allowed Files)
- `.config/nextest.toml` (`[profile.ci] test-threads = 1`)
- `docs/test-migration-race-design.md`

The earlier `crates/db_schema_setup/src/lib.rs` change is reverted to match
`main`.

## Verification
- CI is the real test (needs Postgres + the nextest runner). Expect the
  concurrency-caused failures (the `pg_type` race and the `left==right`
  assertions) to disappear.
- Caveat: a separate, documented, **deterministic** failure — the deprecated
  `PersonInsertForm::test_form` seeding a NULL `wallet_id` — is NOT a concurrency
  issue and will remain until fixed on its own. So this change should make the
  suite *much* greener but may not make the job fully green by itself.
