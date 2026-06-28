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
