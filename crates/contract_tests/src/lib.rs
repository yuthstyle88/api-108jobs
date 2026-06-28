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
  use app_108jobs_db_schema::newtypes::{Coin, WalletId};
  use app_108jobs_db_views_wallet::api::{GetWalletResponse, WalletOperationResponse};

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
    assert!(
      j.get("walletId").is_some(),
      "walletId missing — Flutter breaks"
    );
    assert!(
      j.get("balance").is_some(),
      "balance missing — Flutter breaks"
    );
    assert!(
      j.get("escrowBalance").is_some(),
      "escrowBalance missing — Flutter breaks"
    );
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
    assert!(
      j.get("transactionAmount").is_some(),
      "transactionAmount missing"
    );
    assert!(j.get("success").is_some(), "success missing");
  }
}

#[cfg(test)]
mod auth {
  use app_108jobs_db_schema::sensitive::SensitiveString;
  use app_108jobs_db_views_site::api::LoginResponse;

  #[test]
  fn auth_login_response_has_jwt() {
    // Flutter AuthApi.login reads: jwt, registrationCreated, verifyEmailSent, acceptedTerms
    // LoginResponse uses rename_all = "camelCase" and #[skip_serializing_none].
    // jwt is Option<SensitiveString>; we provide Some(...) so the key appears in JSON.
    // NOTE: LoginResponse does NOT include refreshToken — gap documented, test pins what exists.
    let resp = LoginResponse {
      jwt: Some(SensitiveString::from(String::from("test_token"))),
      registration_created: false,
      verify_email_sent: false,
      accepted_terms: false,
    };
    let j = serde_json::to_value(&resp).expect("serialise");
    // jwt is present because we supplied Some(...)
    assert!(
      j.get("jwt").is_some(),
      "jwt key missing — Flutter AuthApi breaks"
    );
    // Remaining fields must be camelCase (rename_all = "camelCase")
    assert!(
      j.get("registrationCreated").is_some(),
      "registrationCreated missing"
    );
    assert!(
      j.get("verifyEmailSent").is_some(),
      "verifyEmailSent missing"
    );
    assert!(j.get("acceptedTerms").is_some(), "acceptedTerms missing");
    // Negative: snake_case keys must not appear
    assert!(
      j.get("registration_created").is_none(),
      "snake_case key leaked"
    );
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
    let resp: WorkFlowOperationResponse = serde_json::from_value(serde_json::json!({
      "workflowId": 1,
      "status": "InProgress",
      "success": true
    }))
    .expect("WorkFlowOperationResponse fixture parse failed");
    let j = serde_json::to_value(&resp).expect("serialise");
    assert!(
      j.get("workflowId").is_some(),
      "workflowId missing — Flutter breaks"
    );
    assert!(j.get("status").is_some(), "status missing — Flutter breaks");
    assert!(
      j.get("success").is_some(),
      "success missing — Flutter breaks"
    );
    assert!(j.get("workflow_id").is_none(), "snake_case key leaked");
  }
}

#[cfg(test)]
mod bank {
  use app_108jobs_db_schema::newtypes::BankId;
  use app_108jobs_db_views_bank_account::api::BankResponse;

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
    assert!(
      j.get("name").is_some(),
      "name missing — Flutter BankAccountApi breaks"
    );
    assert!(j.get("countryId").is_some(), "countryId missing");
    assert!(j.get("country_id").is_none(), "snake_case key leaked");
  }
}

/// Shared person fixture JSON used by post and comment tests.
/// Fields included: all required (non-Option, non-#[serde(skip)]) fields of Person.
/// Person has rename_all = "camelCase" so all keys here are camelCase.
#[cfg(test)]
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
#[cfg(test)]
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
      "canMod": false,
      "creatorBanned": false,
      "creatorIsModerator": false,
      "creatorBannedFromCategory": false
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
      "canMod": false,
      "creatorBanned": false,
      "creatorIsModerator": false,
      "creatorBannedFromCategory": false
    }))
    .expect("PostView fixture parse failed");

    let j = serde_json::to_value(&pv).expect("serialise");
    let post = j.get("post").expect("post key missing");
    let creator = j.get("creator").expect("creator key missing");

    // post keys (camelCase from Post.rename_all)
    assert!(post.get("id").is_some(), "post.id missing");
    assert!(post.get("name").is_some(), "post.name missing");
    assert!(
      post.get("creatorId").is_some(),
      "post.creatorId missing — Flutter breaks"
    );
    assert!(
      post.get("budget").is_some(),
      "post.budget missing — 108Jobs field"
    );
    assert!(
      post.get("postKind").is_some(),
      "post.postKind missing — 108Jobs field"
    );

    // creator keys (camelCase from Person.rename_all)
    assert!(creator.get("id").is_some(), "creator.id missing");
    assert!(creator.get("name").is_some(), "creator.name missing");
    assert!(
      creator.get("walletId").is_some(),
      "creator.walletId missing — 108Jobs field"
    );
    assert!(
      creator.get("available").is_some(),
      "creator.available missing — 108Jobs field"
    );
  }
}

#[cfg(test)]
mod comment {
  use super::{person_fixture, post_fixture};
  use app_108jobs_db_views_comment::{
    api::{CommentResponse, GetCommentsResponse},
    CommentView,
  };

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
    let cv: CommentView =
      serde_json::from_value(comment_view_fixture()).expect("CommentView fixture parse failed");
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
    let cv: CommentView =
      serde_json::from_value(comment_view_fixture()).expect("CommentView fixture parse failed");
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
    assert!(
      arr[0].get("comment").is_some(),
      "comment nested object missing"
    );
    assert!(
      arr[0].get("creator").is_some(),
      "creator nested object missing"
    );
  }
}

#[cfg(test)]
mod category {
  use app_108jobs_db_views_category::{api::ListCategoriesResponse, CategoryView};

  fn category_fixture() -> serde_json::Value {
    // Category has rename_all = "camelCase".
    // Required non-Option non-serde(skip) fields:
    // id, name, title, removed, publishedAt, deleted, selfPromotion, apId, local,
    // postingRestrictedToMods, instanceId, visibility, subscribers, posts, comments,
    // usersActiveDay, usersActiveWeek, usersActiveMonth, usersActiveHalfYear,
    // subscribersLocal, reportCount, unresolvedReportCount, localRemoved, active, isNew.
    // path is cfg(feature="full") non-Option Ltree — included as dot-separated string.
    // last_refreshed_at, followers_url, inbox_url, moderators_url, featured_url,
    // hot_rank, random_number, interactions_month have #[serde(skip)] — excluded.
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
      "isNew": false,
      "path": "0.1"
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

    let cats = j
      .get("categories")
      .expect("categories key missing — Flutter CategoryApi breaks");
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
