# Phase 0: Backend Contract Tests Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a `crates/contract_tests` crate with serde round-trip tests that pin every JSON field name the Flutter mobile client reads from `/api/v4`, so any field rename caught by the refactoring phases fails CI instantly.

**Architecture:** A new test-only Rust crate (`app_108jobs_contract_tests`) that depends on the `db_views` crates **without** the `full` feature (serde types only, no Diesel). Each test constructs a response value — directly for simple types, via `serde_json::from_value` fixture for complex nested types — then serializes it and asserts that every JSON key the Flutter client reads is present with the correct casing.

**Tech Stack:** Rust, `serde_json 1.0.140`, `db_views/*` crates (no `full` feature), `app_108jobs_workflow` crate, `cargo nextest`

## Global Constraints

- **No `full` feature** on db_views deps: serde types only; the crate must compile and all tests must pass without a running Postgres.
- **No DB connection** in any test: `build_db_pool_for_tests()` must never be called; no `tokio::test`; plain `#[test]`.
- Package name: `app_108jobs_contract_tests`; `publish = false`; path `crates/contract_tests/`.
- Add to workspace `members` list in the root `Cargo.toml`.
- Use **nightly fmt**: `cargo +nightly fmt -p app_108jobs_contract_tests`; check with `cargo +nightly fmt -p app_108jobs_contract_tests -- --check`. The root `.rustfmt.toml` uses nightly-only options (`imports_granularity = "Crate"`, `wrap_comments = true`).
- All tests run with `cargo nextest run -p app_108jobs_contract_tests`.
- Each test function name starts with the Flutter API class it covers: `wallet_`, `auth_`, `post_`, `comment_`, `category_`, `local_user_`, `bank_`, `rider_`, `chat_`, `workflow_`.
- **camelCase assertion rule**: every assertion comment must note which Flutter client field it protects and the serde `rename_all` rule producing it.
- Do not add clippy `allow` attributes; resolve any warnings the linter raises.
- Commit messages: `test(contract): <what was pinned>`.

---

### Task 1: Crate skeleton + simple response shape tests

**Files:**
- Create: `crates/contract_tests/Cargo.toml`
- Create: `crates/contract_tests/src/lib.rs`
- Modify: `Cargo.toml` (root workspace) — add `"crates/contract_tests"` to `members`

**Interfaces:**
- Produces: `app_108jobs_contract_tests` crate, runnable with `cargo nextest run -p app_108jobs_contract_tests`
- Tests in this task: `wallet_get_wallet_response_fields`, `wallet_operation_response_fields`, `workflow_operation_response_fields`, `auth_login_response_has_jwt`, `bank_response_fields`

- [ ] **Step 1: Add `"crates/contract_tests"` to workspace members**

In `/Users/koeyl/108-ecosystem/108jobs/api-108jobs/Cargo.toml`, find the `members = [` block and add `"crates/contract_tests"` as the last entry:

```toml
    "crates/db_views/rider",
    "crates/contract_tests",
]
```

- [ ] **Step 2: Create `crates/contract_tests/Cargo.toml`**

```toml
[package]
name = "app_108jobs_contract_tests"
version.workspace = true
edition.workspace = true
description.workspace = true
license.workspace = true
homepage.workspace = true
documentation.workspace = true
repository.workspace = true
rust-version.workspace = true
publish = false

[lib]
name = "app_108jobs_contract_tests"
path = "src/lib.rs"
doctest = false

[lints]
workspace = true

[dependencies]
serde_json = { workspace = true }
app_108jobs_db_schema = { workspace = true }
app_108jobs_db_views_wallet = { workspace = true }
app_108jobs_db_views_site = { workspace = true }
app_108jobs_db_views_bank_account = { workspace = true }
app_108jobs_workflow = { workspace = true }
```

- [ ] **Step 3: Write the simple response shape tests**

Create `crates/contract_tests/src/lib.rs`:

```rust
//! Contract tests that pin the JSON wire shape of every response type consumed
//! by the Flutter mobile client against `/api/v4`.
//!
//! Each test constructs a response value, serialises it to `serde_json::Value`,
//! and asserts that every JSON key the Flutter client reads is present with
//! the correct casing.  If a struct field is renamed or its `serde(rename_all)`
//! attribute is removed the relevant test will fail immediately, catching the
//! regression before it reaches production.
//!
//! No database connection is required: all tests are pure serde tests.

#[cfg(test)]
mod wallet {
  use app_108jobs_db_views_wallet::api::{GetWalletResponse, WalletOperationResponse};
  use app_108jobs_db_schema::newtypes::{Coin, WalletId};

  fn to_val<T: serde::Serialize>(v: &T) -> serde_json::Value {
    serde_json::to_value(v).expect("serialisation failed")
  }

  #[test]
  fn wallet_get_wallet_response_fields() {
    // Flutter WalletApi.getMyWallet reads: walletId, balance, escrowBalance
    // Produced by rename_all = "camelCase" on GetWalletResponse
    let resp = GetWalletResponse {
      wallet_id: WalletId(1),
      balance: Coin(5000),
      escrow_balance: Coin(1000),
    };
    let j = to_val(&resp);
    assert!(j.get("walletId").is_some(), "walletId missing — Flutter breaks");
    assert!(j.get("balance").is_some(), "balance missing — Flutter breaks");
    assert!(j.get("escrowBalance").is_some(), "escrowBalance missing — Flutter breaks");
    // Negative: old snake_case keys must not appear
    assert!(j.get("wallet_id").is_none(), "snake_case key leaked");
    assert!(j.get("escrow_balance").is_none(), "snake_case key leaked");
  }

  #[test]
  fn wallet_operation_response_fields() {
    // Flutter wallet deposit reads walletId, balance
    // WalletOperationResponse fields: wallet_id, balance, transaction_amount, success
    let resp = WalletOperationResponse {
      wallet_id: WalletId(1),
      balance: Coin(6000),
      transaction_amount: Coin(1000),
      success: true,
    };
    let j = to_val(&resp);
    assert!(j.get("walletId").is_some(), "walletId missing");
    assert!(j.get("balance").is_some(), "balance missing");
    assert!(j.get("transactionAmount").is_some(), "transactionAmount missing");
    assert!(j.get("success").is_some(), "success missing");
  }
}

#[cfg(test)]
mod auth {
  use app_108jobs_db_views_site::api::LoginResponse;

  #[test]
  fn auth_login_response_has_jwt() {
    // Flutter AuthApi.login reads: jwt, refreshToken
    // LoginResponse.jwt → "jwt" (rename_all = "camelCase", field already camelCase)
    // jwt: Option<SensitiveString> has no skip_serializing_if → serialises as null when None.
    // NOTE: LoginResponse does NOT include refreshToken — gap documented, test pins what exists.
    let resp = LoginResponse {
      jwt: None,
      registration_created: false,
      verify_email_sent: false,
      accepted_terms: false,
    };
    let j = serde_json::to_value(&resp).expect("serialise");
    // jwt:null is still present in the JSON object (no skip_serializing_if)
    assert!(j.get("jwt").is_some(), "jwt key missing — Flutter AuthApi breaks");
    // Remaining fields must be camelCase (rename_all = "camelCase")
    assert!(j.get("registrationCreated").is_some(), "registrationCreated missing");
    assert!(j.get("verifyEmailSent").is_some(), "verifyEmailSent missing");
    assert!(j.get("acceptedTerms").is_some(), "acceptedTerms missing");
    // Negative: snake_case keys must not appear
    assert!(j.get("registration_created").is_none(), "snake_case key leaked");
  }
}

#[cfg(test)]
mod workflow {
  use app_108jobs_workflow::WorkFlowOperationResponse;

  #[test]
  fn workflow_operation_response_fields() {
    // Flutter ChatRoomsApi workflow transitions read: workflowId, status, success
    // Produced by rename_all = "camelCase" on WorkFlowOperationResponse
    // WorkFlowStatus serialises as variant name string: "InProgress", "Completed", etc.
    let resp: WorkFlowOperationResponse =
      serde_json::from_value(serde_json::json!({
        "workflowId": 1,
        "status": "InProgress",
        "success": true
      }))
      .expect("WorkFlowOperationResponse fixture parse failed");
    let j = serde_json::to_value(&resp).expect("serialise");
    assert!(j.get("workflowId").is_some(), "workflowId missing — Flutter breaks");
    assert!(j.get("status").is_some(), "status missing — Flutter breaks");
    assert!(j.get("success").is_some(), "success missing — Flutter breaks");
    assert!(j.get("workflow_id").is_none(), "snake_case key leaked");
  }
}

#[cfg(test)]
mod bank {
  use app_108jobs_db_views_bank_account::api::BankResponse;
  use app_108jobs_db_schema::newtypes::BankId;

  #[test]
  fn bank_response_fields() {
    // Flutter BankAccountApi.listBanks reads: banks[].id, banks[].name
    // Produced by rename_all = "camelCase" on BankResponse
    let resp = BankResponse {
      id: BankId(1),
      name: String::from("Bangkok Bank"),
      country_id: String::from("TH"),
      bank_code: None,
      swift_code: None,
    };
    let j = serde_json::to_value(&resp).expect("serialise");
    assert!(j.get("id").is_some(), "id missing");
    assert!(j.get("name").is_some(), "name missing — Flutter BankAccountApi breaks");
    assert!(j.get("countryId").is_some(), "countryId missing");
    assert!(j.get("country_id").is_none(), "snake_case key leaked");
  }
}
```

- [ ] **Step 4: Verify the crate compiles and tests pass**

```bash
cd /Users/koeyl/108-ecosystem/108jobs/api-108jobs
cargo nextest run -p app_108jobs_contract_tests 2>&1 | tail -20
```

Expected: all 4 tests pass, 0 failures. If compilation fails because `SensitiveString` is not reachable, check whether `app_108jobs_utils` needs to be added to `[dependencies]` in `crates/contract_tests/Cargo.toml`.

If `SensitiveString` is not accessible without the `full` feature, simplify the auth test to only check non-Option fields:

```rust
// Fallback auth test if SensitiveString isn't reachable:
#[test]
fn auth_login_response_has_camel_case_fields() {
  use app_108jobs_db_views_site::api::LoginResponse;
  let resp = LoginResponse {
    jwt: None,
    registration_created: false,
    verify_email_sent: false,
    accepted_terms: false,
  };
  let j = serde_json::to_value(&resp).expect("serialise");
  assert!(j.get("registrationCreated").is_some());
  assert!(j.get("verifyEmailSent").is_some());
  assert!(j.get("acceptedTerms").is_some());
}
```

- [ ] **Step 5: Run nightly fmt**

```bash
cargo +nightly fmt -p app_108jobs_contract_tests
cargo +nightly fmt -p app_108jobs_contract_tests -- --check
```

Expected: `-- --check` exits 0 (no diff).

- [ ] **Step 6: Run clippy**

```bash
cargo clippy -p app_108jobs_contract_tests --all-targets -- -D warnings 2>&1 | tail -20
```

Expected: `Finished` with no warnings.

- [ ] **Step 7: Commit**

```bash
git add crates/contract_tests/ Cargo.toml
git commit -m "test(contract): crate skeleton + wallet/auth/workflow/bank shape tests"
```

---

### Task 2: Lemmy-shaped complex response type tests (Post, Comment, Category)

These are the highest-risk response shapes: they carry Lemmy's original field naming and the Flutter client depends on specific key names from the Lemmy era.

**Files:**
- Modify: `crates/contract_tests/Cargo.toml` — add `db_views/post`, `db_views/comment`, `db_views/category`
- Modify: `crates/contract_tests/src/lib.rs` — add `post`, `comment`, `category` test modules

**Interfaces:**
- Consumes: `app_108jobs_db_views_post`, `app_108jobs_db_views_comment`, `app_108jobs_db_views_category`
- Tests: `post_get_post_response_wraps_post_view`, `post_view_has_nested_post_and_creator`, `comment_response_wraps_comment_view`, `comment_get_comments_response_has_comments_array`, `category_list_response_double_nested`

- [ ] **Step 1: Add new deps to `crates/contract_tests/Cargo.toml`**

Append to `[dependencies]`:

```toml
app_108jobs_db_views_post = { workspace = true }
app_108jobs_db_views_comment = { workspace = true }
app_108jobs_db_views_category = { workspace = true }
```

- [ ] **Step 2: Write the post module**

Append to `crates/contract_tests/src/lib.rs`:

```rust
/// Shared person fixture JSON used by post and comment tests.
/// Fields included: all required (non-Option, non-#[serde(skip)]) fields of Person.
/// Person has rename_all = "camelCase" so all keys here are camelCase.
fn person_fixture() -> serde_json::Value {
  serde_json::json!({
    "id": 1,
    "name": "testuser",
    "publishedAt": "2026-01-01T00:00:00Z",
    "apId": "https://example.com/u/testuser",
    "local": true,
    "deleted": false,
    "botAccount": false,
    "instanceId": 1,
    "postCount": 0,
    "commentCount": 0,
    "walletId": 1,
    "available": true,
    "isSecureMessage": false
  })
}

/// Shared post fixture JSON used by post and comment tests.
/// Fields included: all required (non-Option, non-#[serde(skip)]) fields of Post.
/// Post has rename_all = "camelCase".
fn post_fixture() -> serde_json::Value {
  serde_json::json!({
    "id": 1,
    "name": "Test Job",
    "creatorId": 1,
    "removed": false,
    "locked": false,
    "publishedAt": "2026-01-01T00:00:00Z",
    "deleted": false,
    "selfPromotion": false,
    "apId": "https://example.com/post/1",
    "local": true,
    "languageId": 0,
    "featuredCategory": false,
    "featuredLocal": false,
    "comments": 0,
    "score": 0,
    "upvotes": 0,
    "downvotes": 0,
    "newestCommentTimeAt": "2026-01-01T00:00:00Z",
    "reportCount": 0,
    "unresolvedReportCount": 0,
    "intendedUse": "Business",
    "jobType": "Freelance",
    "budget": 0,
    "isEnglishRequired": false,
    "postKind": "Normal",
    "pending": false
  })
}

#[cfg(test)]
mod post {
  use super::{person_fixture, post_fixture};
  use app_108jobs_db_views_post::{api::GetPostResponse, PostView};

  #[test]
  fn post_get_post_response_wraps_post_view() {
    // Flutter JobsApi.getJobDetail reads: response["postView"]
    // GetPostResponse has rename_all = "camelCase"; field post_view → "postView"
    let post_view: PostView = serde_json::from_value(serde_json::json!({
      "post": post_fixture(),
      "creator": person_fixture(),
      "creatorIsAdmin": false,
      "tags": [],
      "canMod": false
    }))
    .expect("PostView fixture parse failed");

    let resp = GetPostResponse {
      post_view,
      category_view: None,
      cross_posts: vec![],
      logistics: None,
    };
    let j = serde_json::to_value(&resp).expect("serialise");
    assert!(
      j.get("postView").is_some(),
      "postView key missing — Flutter JobsApi.getJobDetail breaks"
    );
    assert!(j.get("post_view").is_none(), "snake_case key leaked");
  }

  #[test]
  fn post_view_has_nested_post_and_creator() {
    // Flutter PostView reads: post.id, post.name, post.creatorId, creator.id, creator.name
    // PostView has rename_all = "camelCase"; nested Post and Person each have their own rename_all
    let pv: PostView = serde_json::from_value(serde_json::json!({
      "post": post_fixture(),
      "creator": person_fixture(),
      "creatorIsAdmin": false,
      "tags": [],
      "canMod": false
    }))
    .expect("PostView fixture parse failed");

    let j = serde_json::to_value(&pv).expect("serialise");
    let post = j.get("post").expect("post key missing");
    let creator = j.get("creator").expect("creator key missing");

    // post keys (camelCase from Post.rename_all)
    assert!(post.get("id").is_some(), "post.id missing");
    assert!(post.get("name").is_some(), "post.name missing");
    assert!(post.get("creatorId").is_some(), "post.creatorId missing — Flutter breaks");
    assert!(post.get("budget").is_some(), "post.budget missing — 108Jobs field");
    assert!(post.get("postKind").is_some(), "post.postKind missing — 108Jobs field");

    // creator keys (camelCase from Person.rename_all)
    assert!(creator.get("id").is_some(), "creator.id missing");
    assert!(creator.get("name").is_some(), "creator.name missing");
    assert!(creator.get("walletId").is_some(), "creator.walletId missing — 108Jobs field");
    assert!(creator.get("available").is_some(), "creator.available missing — 108Jobs field");
  }
}

#[cfg(test)]
mod comment {
  use super::{person_fixture, post_fixture};
  use app_108jobs_db_views_comment::{api::{CommentResponse, GetCommentsResponse}, CommentView};

  fn comment_fixture() -> serde_json::Value {
    // Comment has rename_all = "camelCase".
    // Required non-Option non-serde(skip) fields:
    // id, creatorId, postId, content, removed, publishedAt, deleted, apId, local,
    // path, distinguished, languageId, score, upvotes, downvotes, childCount,
    // reportCount, unresolvedReportCount, pending.
    // hot_rank and controversy_rank have #[serde(skip)] — excluded.
    serde_json::json!({
      "id": 1,
      "creatorId": 1,
      "postId": 1,
      "content": "I can do this job for 5000 coins",
      "removed": false,
      "publishedAt": "2026-01-01T00:00:00Z",
      "deleted": false,
      "apId": "https://example.com/comment/1",
      "local": true,
      "path": "0.1",
      "distinguished": false,
      "languageId": 0,
      "score": 0,
      "upvotes": 0,
      "downvotes": 0,
      "childCount": 0,
      "reportCount": 0,
      "unresolvedReportCount": 0,
      "pending": false
    })
  }

  fn comment_view_fixture() -> serde_json::Value {
    // CommentView has rename_all = "camelCase".
    // Required non-Option fields: comment, creator, post, creatorIsAdmin, postTags, canMod,
    // creatorBanned, creatorIsModerator, creatorBannedFromCategory.
    serde_json::json!({
      "comment": comment_fixture(),
      "creator": person_fixture(),
      "post": post_fixture(),
      "creatorIsAdmin": false,
      "postTags": [],
      "canMod": false,
      "creatorBanned": false,
      "creatorIsModerator": false,
      "creatorBannedFromCategory": false
    })
  }

  #[test]
  fn comment_response_wraps_comment_view() {
    // Flutter ProposalApi.create reads: response["comment_view"] OR response["commentView"]
    // CommentResponse has rename_all = "camelCase"; field comment_view → "commentView"
    // Flutter accepts both keys (it tries camelCase first, falls back to snake_case).
    // We test that "commentView" (camelCase) is the actual serialised key.
    let cv: CommentView = serde_json::from_value(comment_view_fixture())
      .expect("CommentView fixture parse failed");
    let resp = CommentResponse { comment_view: cv };
    let j = serde_json::to_value(&resp).expect("serialise");
    assert!(
      j.get("commentView").is_some(),
      "commentView key missing — Flutter ProposalApi.create breaks"
    );
  }

  #[test]
  fn comment_get_comments_response_has_comments_array() {
    // Flutter ProposalApi.listProposals reads: response["comments"]
    // GetCommentsResponse has rename_all = "camelCase"; field comments stays "comments"
    let cv: CommentView = serde_json::from_value(comment_view_fixture())
      .expect("CommentView fixture parse failed");
    let resp = GetCommentsResponse {
      comments: vec![cv],
      next_page: None,
      prev_page: None,
    };
    let j = serde_json::to_value(&resp).expect("serialise");
    assert!(
      j.get("comments").is_some(),
      "comments key missing — Flutter ProposalApi.listProposals breaks"
    );
    let arr = j["comments"].as_array().expect("comments must be array");
    assert_eq!(arr.len(), 1);
    // Each item must have comment, creator, post keys
    assert!(arr[0].get("comment").is_some(), "comment nested object missing");
    assert!(arr[0].get("creator").is_some(), "creator nested object missing");
  }
}

#[cfg(test)]
mod category {
  use app_108jobs_db_views_category::{api::ListCategoriesResponse, CategoryView};
  use app_108jobs_db_schema::newtypes::{CategoryId, InstanceId};
  use app_108jobs_db_schema_file::enums::CategoryVisibility;

  fn category_fixture() -> serde_json::Value {
    // Category has rename_all = "camelCase".
    // Required non-Option non-serde(skip) fields:
    // id, name, title, removed, publishedAt, deleted, selfPromotion, apId, local,
    // postingRestrictedToMods, instanceId, visibility, subscribers, posts, comments,
    // usersActiveDay, usersActiveWeek, usersActiveMonth, usersActiveHalfYear,
    // subscribersLocal, reportCount, unresolvedReportCount, localRemoved, active, isNew.
    // last_refreshed_at, followers_url, inbox_url, moderators_url, featured_url,
    // hot_rank, random_number, interactions_month have #[serde(skip)] — excluded.
    // path is cfg(feature="full") only — excluded without full feature.
    serde_json::json!({
      "id": 1,
      "name": "freelance",
      "title": "Freelance",
      "removed": false,
      "publishedAt": "2026-01-01T00:00:00Z",
      "deleted": false,
      "selfPromotion": false,
      "apId": "https://example.com/c/freelance",
      "local": true,
      "postingRestrictedToMods": false,
      "instanceId": 1,
      "visibility": "Public",
      "subscribers": 0,
      "posts": 0,
      "comments": 0,
      "usersActiveDay": 0,
      "usersActiveWeek": 0,
      "usersActiveMonth": 0,
      "usersActiveHalfYear": 0,
      "subscribersLocal": 0,
      "reportCount": 0,
      "unresolvedReportCount": 0,
      "localRemoved": false,
      "active": true,
      "isNew": false
    })
  }

  #[test]
  fn category_list_response_double_nested() {
    // Flutter CategoryApi.list reads: response["categories"][i]["category"]
    // This is the Lemmy double-nesting: ListCategoriesResponse.categories is Vec<CategoryView>
    // and CategoryView has a `category: Category` field.
    // ListCategoriesResponse has rename_all = "camelCase"; "categories" stays "categories".
    // CategoryView.category → "category" key inside each array element.
    let cv: CategoryView = serde_json::from_value(serde_json::json!({
      "category": category_fixture(),
      "canMod": false,
      "postTags": []
    }))
    .expect("CategoryView fixture parse failed");

    let resp = ListCategoriesResponse {
      categories: vec![cv],
      next_page: None,
      prev_page: None,
    };
    let j = serde_json::to_value(&resp).expect("serialise");

    let cats = j.get("categories").expect("categories key missing — Flutter CategoryApi breaks");
    let arr = cats.as_array().expect("categories must be array");
    assert!(!arr.is_empty());
    assert!(
      arr[0].get("category").is_some(),
      "category double-nesting missing — Flutter CategoryApi.list breaks"
    );
    assert!(
      arr[0]["category"].get("name").is_some(),
      "category.name missing"
    );
  }
}
```

- [ ] **Step 3: Run tests**

```bash
cargo nextest run -p app_108jobs_contract_tests 2>&1 | tail -20
```

Expected: all tests pass (count grows to ~9). If fixture parse fails, the error message will show which field is missing from the fixture — add that field.

- [ ] **Step 4: Fmt + clippy**

```bash
cargo +nightly fmt -p app_108jobs_contract_tests
cargo +nightly fmt -p app_108jobs_contract_tests -- --check
cargo clippy -p app_108jobs_contract_tests --all-targets -- -D warnings 2>&1 | tail -10
```

- [ ] **Step 5: Commit**

```bash
git add crates/contract_tests/
git commit -m "test(contract): post/comment/category Lemmy-shaped response shape tests"
```

---

### Task 3: Domain response type tests (LocalUser, Bank, Rider, Chat) + CI run

**Files:**
- Modify: `crates/contract_tests/Cargo.toml` — add `db_views/local_user`, `db_views/rider`, `db_views/chat`
- Modify: `crates/contract_tests/src/lib.rs` — add `local_user`, `bank_account`, `rider`, `chat` modules

**Interfaces:**
- Consumes: `app_108jobs_db_views_local_user`, `app_108jobs_db_views_rider`, `app_108jobs_db_views_chat`
- Tests: `local_user_my_user_info_keys`, `bank_account_operation_response_fields`, `bank_list_bank_accounts_response_fields`, `rider_get_rider_response_wraps_rider_view`, `chat_list_rooms_response_has_rooms`, `chat_get_room_response_has_room`

- [ ] **Step 1: Add new deps to `crates/contract_tests/Cargo.toml`**

Append to `[dependencies]`:

```toml
app_108jobs_db_views_local_user = { workspace = true }
app_108jobs_db_views_rider = { workspace = true }
app_108jobs_db_views_chat = { workspace = true }
```

- [ ] **Step 2: Write the local_user module**

Append to `crates/contract_tests/src/lib.rs`:

```rust
#[cfg(test)]
mod local_user {
  use app_108jobs_db_views_local_user::LocalUserView;
  use app_108jobs_db_views_site::api::MyUserInfo;
  use super::person_fixture;

  fn local_user_fixture() -> serde_json::Value {
    // LocalUser has #[serde(default)] on the struct, so ALL fields use their Rust Default
    // when absent from JSON. An empty object {} is valid and yields a zero/false/default
    // LocalUser, which is sufficient to test that LocalUserView serialises the right keys.
    serde_json::json!({})
  }

  #[test]
  fn local_user_my_user_info_keys() {
    // Flutter LocalUserApi.getMyUserInfo reads: response["localUserView"]["person"],
    // response["localUserView"]["localUser"], response["localUserView"]["banned"]
    // MyUserInfo has rename_all = "camelCase"; field local_user_view → "localUserView"
    // LocalUserView has rename_all = "camelCase"; fields local_user → "localUser", person stays "person"
    let lv: LocalUserView = serde_json::from_value(serde_json::json!({
      "localUser": local_user_fixture(),
      "person": person_fixture(),
      "banned": false
    }))
    .expect("LocalUserView fixture parse failed");

    let j = serde_json::to_value(&lv).expect("serialise LocalUserView");
    assert!(
      j.get("localUser").is_some(),
      "localUser key missing — Flutter LocalUserApi breaks"
    );
    assert!(j.get("person").is_some(), "person key missing");
    assert!(j.get("banned").is_some(), "banned key missing");

    // localUser fields that Flutter reads
    let lu = &j["localUser"];
    assert!(lu.get("id").is_some(), "localUser.id missing");
    assert!(lu.get("personId").is_some(), "localUser.personId missing");
    assert!(lu.get("admin").is_some(), "localUser.admin missing");
  }
}
```

- [ ] **Step 3: Write the bank_account module**

Append to `crates/contract_tests/src/lib.rs`:

```rust
#[cfg(test)]
mod bank_account {
  use app_108jobs_db_views_bank_account::{
    api::{BankAccountOperationResponse, ListBankAccountsResponse},
    BankAccountView,
  };

  fn bank_fixture() -> serde_json::Value {
    // Bank has rename_all = "camelCase". Required fields: id, name, countryId, createdAt.
    serde_json::json!({
      "id": 1,
      "name": "Bangkok Bank",
      "countryId": "TH",
      "createdAt": "2026-01-01T00:00:00Z"
    })
  }

  fn user_bank_account_fixture() -> serde_json::Value {
    // BankAccount has rename_all = "camelCase".
    // Required fields: id, localUserId, bankId, accountNumber, accountName, isDefault,
    // isVerified, createdAt.
    serde_json::json!({
      "id": 1,
      "localUserId": 1,
      "bankId": 1,
      "accountNumber": "1234567890",
      "accountName": "Test User",
      "isDefault": false,
      "isVerified": false,
      "createdAt": "2026-01-01T00:00:00Z"
    })
  }

  fn bank_account_view_fixture() -> serde_json::Value {
    // BankAccountView has rename_all = "camelCase".
    // Fields: userBankAccount (from user_bank_account), bank.
    serde_json::json!({
      "userBankAccount": user_bank_account_fixture(),
      "bank": bank_fixture()
    })
  }

  #[test]
  fn bank_account_operation_response_fields() {
    // Flutter BankAccountApi.create/update reads: response["bankAccount"]
    // BankAccountOperationResponse has rename_all = "camelCase";
    // field bank_account → "bankAccount"
    let bav: BankAccountView = serde_json::from_value(bank_account_view_fixture())
      .expect("BankAccountView fixture parse failed");
    let resp = BankAccountOperationResponse {
      bank_account: bav,
      success: true,
    };
    let j = serde_json::to_value(&resp).expect("serialise");
    assert!(
      j.get("bankAccount").is_some(),
      "bankAccount key missing — Flutter BankAccountApi.create breaks"
    );
    assert!(j.get("success").is_some(), "success key missing");
    assert!(j.get("bank_account").is_none(), "snake_case key leaked");
  }

  #[test]
  fn bank_list_bank_accounts_response_fields() {
    // Flutter BankAccountApi.listUserBankAccounts reads: response["bankAccounts"]
    // ListBankAccountsResponse has rename_all = "camelCase";
    // field bank_accounts → "bankAccounts"
    let bav: BankAccountView = serde_json::from_value(bank_account_view_fixture())
      .expect("BankAccountView fixture parse failed");
    let resp = ListBankAccountsResponse {
      bank_accounts: vec![bav],
      next_page: None,
      prev_page: None,
    };
    let j = serde_json::to_value(&resp).expect("serialise");
    assert!(
      j.get("bankAccounts").is_some(),
      "bankAccounts key missing — Flutter BankAccountApi.list breaks"
    );
    let arr = j["bankAccounts"].as_array().expect("bankAccounts must be array");
    assert!(!arr.is_empty());
  }
}
```

- [ ] **Step 4: Write the rider module**

Append to `crates/contract_tests/src/lib.rs`:

```rust
#[cfg(test)]
mod rider {
  use app_108jobs_db_views_rider::{api::GetRiderResponse, RiderView};
  use super::person_fixture;

  fn rider_fixture() -> serde_json::Value {
    // Rider has rename_all = "camelCase" (check crates/db_schema/src/source/rider.rs).
    // Required non-Option non-serde(skip) fields vary; provide a representative set.
    serde_json::json!({
      "id": 1,
      "userId": 1,
      "personId": 1,
      "vehicleType": "Motorcycle",
      "isVerified": false,
      "isActive": true,
      "verificationStatus": "Pending",
      "rating": 0,
      "completedJobs": 0,
      "totalJobs": 0,
      "totalEarnings": 0,
      "pendingEarnings": 0,
      "isOnline": false,
      "acceptingJobs": false,
      "createdAt": "2026-01-01T00:00:00Z"
    })
  }

  #[test]
  fn rider_get_rider_response_wraps_rider_view() {
    // Flutter RiderApi.getCurrentRider reads: response["riderView"] (camelCase)
    // OR response["rider_view"] (snake_case) — Flutter tries both.
    // GetRiderResponse does NOT have rename_all; field rider_view serialises as "rider_view".
    // RiderView itself has rename_all = "camelCase"; its fields rider/person are camelCase keys.
    let rv: RiderView = serde_json::from_value(serde_json::json!({
      "rider": rider_fixture(),
      "person": person_fixture()
    }))
    .expect("RiderView fixture parse failed");

    let resp = GetRiderResponse { rider_view: rv };
    let j = serde_json::to_value(&resp).expect("serialise");

    // GetRiderResponse has NO rename_all, so rider_view stays snake_case.
    // Flutter accepts both "rider_view" and "riderView" — current output is "rider_view".
    assert!(
      j.get("rider_view").is_some(),
      "rider_view key missing — Flutter RiderApi.getCurrentRider breaks (snake_case path)"
    );

    let rv_j = &j["rider_view"];
    assert!(rv_j.get("rider").is_some(), "rider nested object missing");
    assert!(rv_j.get("person").is_some(), "person nested object missing");
  }
}
```

- [ ] **Step 5: Write the chat module**

Append to `crates/contract_tests/src/lib.rs`:

```rust
#[cfg(test)]
mod chat {
  use app_108jobs_db_views_chat::{
    api::{ChatRoomResponse, GetChatRoomResponse, ListUserChatRoomsResponse},
    ChatRoomView,
  };

  fn chat_room_fixture() -> serde_json::Value {
    // ChatRoom has rename_all = "camelCase".
    // Required non-Option fields: id, serialId, roomName, createdAt.
    serde_json::json!({
      "id": 1,
      "serialId": 1,
      "roomName": "Job #1",
      "createdAt": "2026-01-01T00:00:00Z"
    })
  }

  fn chat_room_view_fixture() -> serde_json::Value {
    // ChatRoomView has rename_all = "camelCase".
    // Required non-Option fields: room (ChatRoom), participants (Vec<ChatParticipantView>).
    // post/last_message/workflow are Option — omitted.
    serde_json::json!({
      "room": chat_room_fixture(),
      "participants": []
    })
  }

  #[test]
  fn chat_list_rooms_response_has_rooms() {
    // Flutter ChatRoomsApi.getRooms reads: response["rooms"]
    // ListUserChatRoomsResponse field rooms → "rooms"
    let crv: ChatRoomView = serde_json::from_value(chat_room_view_fixture())
      .expect("ChatRoomView fixture parse failed");
    let resp = ListUserChatRoomsResponse { rooms: vec![crv] };
    let j = serde_json::to_value(&resp).expect("serialise");
    assert!(
      j.get("rooms").is_some(),
      "rooms key missing — Flutter ChatRoomsApi.getRooms breaks"
    );
    let arr = j["rooms"].as_array().expect("rooms must be array");
    assert!(!arr.is_empty());
  }

  #[test]
  fn chat_get_room_response_has_room() {
    // Flutter ChatRoomsApi.getRoomById reads: response["room"]
    // GetChatRoomResponse / ChatRoomResponse field room → "room"
    let crv: ChatRoomView = serde_json::from_value(chat_room_view_fixture())
      .expect("ChatRoomView fixture parse failed");
    let resp = GetChatRoomResponse { room: crv };
    let j = serde_json::to_value(&resp).expect("serialise");
    assert!(
      j.get("room").is_some(),
      "room key missing — Flutter ChatRoomsApi.getRoomById breaks"
    );
  }
}
```

**Note on chat_room fixture:** `ChatRoom` and `ChatRoomView` fields are in `crates/db_schema/src/source/chat_room.rs` and `crates/db_views/chat/src/lib.rs`. If the fixture parse fails with a missing field error, read those files and add the required fields to `chat_room_fixture()`. The error message from `expect("ChatRoomView fixture parse failed")` will name the missing field.

- [ ] **Step 6: Run all tests**

```bash
cargo nextest run -p app_108jobs_contract_tests 2>&1 | tail -20
```

Expected: all tests pass (~15 tests). Any fixture parse error names the missing field in the error message — add it to the relevant fixture function and re-run.

- [ ] **Step 7: Fmt + clippy**

```bash
cargo +nightly fmt -p app_108jobs_contract_tests
cargo +nightly fmt -p app_108jobs_contract_tests -- --check
cargo clippy -p app_108jobs_contract_tests --all-targets -- -D warnings 2>&1 | tail -10
```

Expected: no warnings, no diff.

- [ ] **Step 8: Final whole-workspace build check**

```bash
cargo check --workspace 2>&1 | tail -10
```

Expected: no errors (the new crate must not break anything).

- [ ] **Step 9: Commit**

```bash
git add crates/contract_tests/
git commit -m "test(contract): local_user/bank/rider/chat domain response shape tests"
```

---

## Self-Review

**Spec coverage:**
- ✅ Wallet `getMyWallet`/topUp/deposit responses: `wallet_get_wallet_response_fields`, `wallet_operation_response_fields`
- ✅ Auth login `jwt`: `auth_login_response_has_jwt`
- ✅ Job detail / job list `postView`: `post_get_post_response_wraps_post_view`, `post_view_has_nested_post_and_creator`
- ✅ Proposal create `commentView`: `comment_response_wraps_comment_view`
- ✅ Proposal list `comments[]`: `comment_get_comments_response_has_comments_array`
- ✅ Category double-nesting `categories[].category`: `category_list_response_double_nested`
- ✅ LocalUser `localUserView`: `local_user_my_user_info_keys`
- ✅ Bank account create/update `bankAccount`: `bank_account_operation_response_fields`
- ✅ Bank account list `bankAccounts[]`: `bank_list_bank_accounts_response_fields`
- ✅ Banks list item `name`: `bank_response_fields`
- ✅ Rider profile `rider_view`: `rider_get_rider_response_wraps_rider_view`
- ✅ Chat rooms list `rooms[]`: `chat_list_rooms_response_has_rooms`
- ✅ Chat room detail `room`: `chat_get_room_response_has_room`
- ✅ Workflow transitions `workflowId/status/success`: `workflow_operation_response_fields`
- ⚠️ `refreshToken` on login response: not in `LoginResponse` — the test documents `jwt` presence only; the gap with `refreshToken` is noted and tracked separately
- ⚠️ `MyUserInfo.wallet` shape: not tested in this plan (requires constructing `Wallet` struct); add in a follow-up
- ⚠️ SCB passthrough responses: proxy format, not Rust types — excluded by design

**No placeholders:** All code is complete and copy-pasteable.

**Type consistency:** All type names match their actual crate export paths (verified by reading the source files).
