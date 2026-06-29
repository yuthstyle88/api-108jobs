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
  use app_108jobs_db::newtypes::{Coin, WalletId};
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
  use app_108jobs_db::sensitive::SensitiveString;
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
  use app_108jobs_db::newtypes::BankId;
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
    "proposalCount": 0,
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
    "proposals": 0,
    "score": 0,
    "upvotes": 0,
    "downvotes": 0,
    "newestProposalTimeAt": "2026-01-01T00:00:00Z",
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
mod proposal {
  use super::{person_fixture, post_fixture};
  use app_108jobs_db_views_proposal::{
    api::{GetCommentsResponse, ProposalResponse},
    ProposalView,
  };

  fn proposal_fixture() -> serde_json::Value {
    // Proposal has rename_all = "camelCase".
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
      "apId": "https://example.com/proposal/1",
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

  fn proposal_view_fixture() -> serde_json::Value {
    // ProposalView has rename_all = "camelCase".
    // Required non-Option fields: proposal, creator, post, creatorIsAdmin, postTags, canMod,
    // creatorBanned, creatorIsModerator, creatorBannedFromCategory.
    serde_json::json!({
      "proposal": proposal_fixture(),
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
  fn proposal_response_wraps_proposal_view() {
    // Flutter ProposalApi.create reads: response["proposal_view"] OR response["proposalView"]
    // ProposalResponse has rename_all = "camelCase"; field proposal_view → "proposalView"
    // Flutter accepts both keys (it tries camelCase first, falls back to snake_case).
    // We test that "proposalView" (camelCase) is the actual serialised key.
    let pv: ProposalView =
      serde_json::from_value(proposal_view_fixture()).expect("ProposalView fixture parse failed");
    let resp = ProposalResponse { proposal_view: pv };
    let j = serde_json::to_value(&resp).expect("serialise");
    assert!(
      j.get("proposalView").is_some(),
      "proposalView key missing — Flutter ProposalApi.create breaks"
    );
  }

  #[test]
  fn proposal_get_proposals_response_has_proposals_array() {
    // Flutter ProposalApi.listProposals reads: response["proposals"]
    // GetCommentsResponse has rename_all = "camelCase"; field proposals stays "proposals"
    let pv: ProposalView =
      serde_json::from_value(proposal_view_fixture()).expect("ProposalView fixture parse failed");
    let resp = GetCommentsResponse {
      proposals: vec![pv],
      next_page: None,
      prev_page: None,
    };
    let j = serde_json::to_value(&resp).expect("serialise");
    assert!(
      j.get("proposals").is_some(),
      "proposals key missing — Flutter ProposalApi.listProposals breaks"
    );
    let arr = j["proposals"].as_array().expect("proposals must be array");
    assert_eq!(arr.len(), 1);
    // Each item must have proposal, creator, post keys
    assert!(
      arr[0].get("proposal").is_some(),
      "proposal nested object missing"
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
      "proposals": 0,
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

#[cfg(test)]
mod local_user {
  use super::person_fixture;
  use app_108jobs_db_views_local_user::LocalUserView;

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
    // LocalUserView has rename_all = "camelCase"; fields local_user → "localUser", person stays
    // "person"
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
    let arr = j["bankAccounts"]
      .as_array()
      .expect("bankAccounts must be array");
    assert!(!arr.is_empty());
  }
}

#[cfg(test)]
mod rider {
  use super::person_fixture;
  use app_108jobs_db_views_rider::{api::GetRiderResponse, RiderView};

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

#[cfg(test)]
mod chat {
  use app_108jobs_db_views_chat::{
    api::{GetChatRoomResponse, ListUserChatRoomsResponse},
    ChatRoomView,
  };

  fn chat_room_fixture() -> serde_json::Value {
    // ChatRoom has rename_all = "camelCase".
    // Required non-Option fields: id, serialId, roomName, createdAt.
    // ChatRoomId wraps a String and must be a 16-char hex string or UUID.
    serde_json::json!({
      "id": "0123456789abcdef",
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
    let crv: ChatRoomView =
      serde_json::from_value(chat_room_view_fixture()).expect("ChatRoomView fixture parse failed");
    let resp = ListUserChatRoomsResponse {
      rooms: vec![crv],
      next_page: None,
      prev_page: None,
    };
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
    let crv: ChatRoomView =
      serde_json::from_value(chat_room_view_fixture()).expect("ChatRoomView fixture parse failed");
    let resp = GetChatRoomResponse { room: crv };
    let j = serde_json::to_value(&resp).expect("serialise");
    assert!(
      j.get("room").is_some(),
      "room key missing — Flutter ChatRoomsApi.getRoomById breaks"
    );
  }
}
