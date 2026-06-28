# Phase 3 — Carve 1: `core` Crate

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rename `crates/utils/` → `crates/core/` and the crate name `app_108jobs_utils` → `app_108jobs_core` across the entire workspace. No logic changes — this is a pure rename so the codebase reads "108Jobs" instead of generic "utils".

**Architecture:** Mass find-replace of the crate identifier, then compile-driven cleanup. The internal module structure of the crate stays identical; only the crate name and directory name change.

**Tech Stack:** Rust, Cargo workspace, `sed`, `cargo check`, `cargo nextest`

## Global Constraints

- **No behavior change.** All types, functions, and module paths (e.g., `app_108jobs_core::error::FastJobResult`) work identically to before; only the crate name changes.
- **No table drops, no migrations, no API changes.**
- **Gate:** `cargo check --workspace` exits 0 (zero errors, zero warnings). `cargo nextest run -p app_108jobs_contract_tests` 16/16 pass.
- **Nightly fmt:** `cargo +nightly fmt` after all changes.
- **No new `#[allow(...)]` attributes.**
- Commit message: `refactor(phase-3): rename app_108jobs_utils → app_108jobs_core`.

## Context

Current state:
- Directory: `crates/utils/`
- Crate name: `app_108jobs_utils`
- 38 other Cargo.toml files depend on it
- Internal module paths (e.g., `settings`, `error`, `utils`) are unchanged — only the crate-level name changes

Target state:
- Directory: `crates/core/`
- Crate name: `app_108jobs_core`
- All import statements: `use app_108jobs_utils::*` → `use app_108jobs_core::*`

---

### Task 1: Rename the crate

**Files:**
- Rename directory: `crates/utils/` → `crates/core/`
- Modify: `crates/core/Cargo.toml` — change `name`
- Modify: root `Cargo.toml` — update member path, workspace dep name + path
- Modify: all other `*.toml` files — `app_108jobs_utils` → `app_108jobs_core`
- Modify: all `*.rs` files — `app_108jobs_utils` → `app_108jobs_core`

**Interfaces:**
- Produces: `app_108jobs_core` crate at `crates/core/`, identical API surface to `app_108jobs_utils`

- [ ] **Step 1: Rename the directory**

```bash
mv crates/utils/ crates/core/
```

- [ ] **Step 2: Update `crates/core/Cargo.toml`**

Change:
```toml
name = "app_108jobs_utils"
```
To:
```toml
name = "app_108jobs_core"
```

- [ ] **Step 3: Mass-replace `app_108jobs_utils` → `app_108jobs_core` in all `.toml` files**

```bash
find . -name "*.toml" -not -path "*/target/*" -exec sed -i '' 's/app_108jobs_utils/app_108jobs_core/g' {} +
```

Verify the workspace root `Cargo.toml` was updated:
```bash
grep "app_108jobs_core\|crates/core" Cargo.toml | head -5
```

Expected: `"crates/core"` in members, `app_108jobs_core = { ... path = "./crates/core" }` in workspace.dependencies.

- [ ] **Step 4: Mass-replace `app_108jobs_utils` → `app_108jobs_core` in all `.rs` files**

```bash
find . -name "*.rs" -not -path "*/target/*" -exec sed -i '' 's/app_108jobs_utils/app_108jobs_core/g' {} +
```

Spot-check:
```bash
grep -rn "app_108jobs_utils" . --include="*.rs" --include="*.toml" | grep -v "target/" | head -5
```

Expected: zero matches (or only in doc comments/plan files, which is fine).

- [ ] **Step 5: Compile check**

```bash
cargo check --workspace 2>&1 | grep "^error" | head -20
```

Expected: zero errors. If there are errors, they will be import paths that the sed didn't catch — fix them manually by reading the compiler output.

Common edge case: if any file uses `extern crate app_108jobs_utils;` (rare in edition 2021 Rust), update those too.

- [ ] **Step 6: Run contract tests**

```bash
cargo nextest run -p app_108jobs_contract_tests 2>&1 | tail -3
```

Expected: 16/16 pass.

- [ ] **Step 7: Nightly fmt + clippy**

```bash
cargo +nightly fmt
cargo +nightly fmt -- --check
```

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "refactor(phase-3): rename app_108jobs_utils → app_108jobs_core"
```

---

## Self-Review

**Spec coverage:**
- ✅ `crates/utils/` renamed to `crates/core/`
- ✅ Crate name `app_108jobs_utils` → `app_108jobs_core` everywhere
- ✅ Workspace members + workspace.dependencies updated
- ✅ All 38 dependent Cargo.toml files updated
- ✅ All `use app_108jobs_utils::*` imports updated
- ✅ No internal module structure changed
- ✅ Gate: `cargo check --workspace` + 16/16 contract tests green

**No placeholders.** Compile-driven: any missed reference surfaces as a compiler error.
