use actix_web::web::Data;
use actix_web::HttpResponse;
use lemmy_api_utils::context::FastJobContext;
use lemmy_utils::error::FastJobResult;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use lemmy_db_views_local_user::LocalUserView;

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

pub async fn generate_scb_token(
    context: Data<FastJobContext>,
    _local_user_view: LocalUserView,
) -> FastJobResult<HttpResponse> {
    let scb = context.settings().scb.clone();
    let url = format!("{}v1/oauth/token", scb.url);

    let request_uid = Uuid::new_v4().to_string();

    let body = TokenRequest {
        application_key: scb.api_key,
        application_secret: scb.api_secret,
    };

    let client = Client::new();
    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("resourceOwnerId", body.application_key.clone())
        .header("requestUId", request_uid)
        .header("accept-language", "EN")
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<TokenResponse>()
        .await?;

    Ok(HttpResponse::Ok().json(res))
}

