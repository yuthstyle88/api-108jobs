# Fix test migration race (double-checked locking) — design

- Date: 2026-06-26
- Branch: `fix/test-migration-race`

## Problem

`cargo test` is red on `main`. Many tests panic at DB setup, not in business
logic:

```
db pool missing: FastJobError { message: Unknown(
  "duplicate key value violates unique constraint \"pg_type_typname_nsp_index\"") }
  → db_schema_setup::run → run_selected_migrations
```

`pg_type_typname_nsp_index` is Postgres's catalog index for type names. The
duplicate-key means **two processes ran `CREATE TYPE` for the same enum at the
same time**. The test suite runs many test binaries in parallel (worse since
the switch to nextest in #77), each calling `build_db_pool_for_tests()` →
`db_schema_setup::run()`, all migrating the **one shared CI Postgres**.

## Root cause (TOCTOU)

`run()` (crates/db_schema_setup/src/lib.rs) has a deliberately **lock-free fast
path**: if there are no pending migrations and the `r` schema is in sync, it
returns early without taking `pg_advisory_lock(0)` — so horizontally-scaled
`app_108jobs_server` startups don't serialize on a global lock.

On a **fresh** database every concurrent process sees "migrations pending", so
they all skip the early return and reach the lock. The advisory lock *does*
serialise the migration run — but each waiter then runs `run_selected_migrations`
unconditionally. The check ("are migrations needed?") happened *before* the lock;
the run happens *after*. Classic check-then-act race: process B re-applies what
process A already applied → `CREATE TYPE` collision.

## Fix — double-checked locking

Keep the lock-free fast path (preserves the production startup optimisation), but
when migrations appear necessary, take the lock and **re-check the same
condition under it** before running migrations. If another process applied them
while we waited, return early.

- Extract the "database already up to date?" predicate into a small closure
  (`db_up_to_date`) so the exact same check runs lock-free and under the lock.
- First call: lock-free fast path (unchanged behaviour for already-migrated DBs).
- Take `pg_advisory_lock(0)`.
- Second call under the lock: if now up to date, early-return; else migrate.

The advisory lock is session-scoped and released when `conn` drops at the end of
`run()`, exactly as today.

## Scope (Allowed Files)
- `crates/db_schema_setup/src/lib.rs` (`run()` only)
- `docs/test-migration-race-design.md`

Out of scope: changing the test runner / nextest config; making individual
migrations idempotent.

## Verification
- Local: `cargo check -p app_108jobs_db_schema_setup` compiles clean (no local
  Postgres to run the concurrent test scenario).
- CI (Postgres service + parallel tests) is the real reproduction: the previously
  flaky `cargo test` job should stop failing at DB setup. **This needs CI to
  confirm** — the race cannot be reproduced in this local environment.

Note: this is a concurrency fix in shared migration-setup infra; behaviour for
the single-process / already-migrated paths is unchanged by construction.
