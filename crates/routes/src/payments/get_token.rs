use actix_web::web::Data;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct TokenRequest {
  #[serde(rename = "applicationKey")]
  application_key: String,
  #[serde(rename = "applicationSecret")]
  application_secret: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
  pub status: Status,
  pub data: Option<TokenData>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
  pub code: i32,
  pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenData {
  #[serde(rename = "accessToken")]
  pub access_token: String,
  #[serde(rename = "tokenType")]
  pub token_type: String,
  #[serde(rename = "expiresIn")]
  pub expires_in: i64,
  #[serde(rename = "expiresAt")]
  pub expires_at: Option<i64>,
}

/// Helper to fetch an SCB OAuth token
pub async fn fetch_scb_token(context: &Data<FastJobContext>) -> FastJobResult<String> {
  let scb = context.settings().scb.clone();
  let url = format!("{}v1/oauth/token", scb.url);

  let request_uid = Uuid::new_v4().to_string();

  let body = TokenRequest {
    application_key: scb.api_key.clone(),
    application_secret: scb.api_secret.clone(),
  };

  let client = Client::new();
  let resp = client
    .post(&url)
    .header("Content-Type", "application/json")
    .header("resourceOwnerId", &scb.api_key)
    .header("requestUId", &request_uid)
    .header("accept-language", "EN")
    .json(&body)
    .send()
    .await?;

  let text = resp.text().await?;

  let parsed: TokenResponse = match serde_json::from_str(&text) {
    Ok(r) => r,
    Err(e) => {
      error!("Failed to parse SCB token JSON: {}", e);
      return Err(FastJobErrorType::ReturnedNonJSONResponse.into());
    }
  };

  if let Some(data) = parsed.data {
    info!(
      "SCB token fetched successfully (expires in {}s)",
      data.expires_in
    );
    Ok(data.access_token)
  } else {
    error!("SCB token response missing data field: {:?}", parsed.status);
    Err(FastJobErrorType::ExternalApiError.into())
  }
}
