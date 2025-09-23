use actix_web::HttpResponse;
use actix_web::web::{Data, Json};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QrCodeRequest {
    body: QrCodeBody,
    token: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QrCodeBody {
    #[serde(rename = "qrType")]
    qr_type: String,
    amount: String,

    // QR 30
    #[serde(rename = "ppType", skip_serializing_if = "Option::is_none")]
    pp_type: Option<String>,
    #[serde(rename = "ppId", skip_serializing_if = "Option::is_none")]
    pp_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ref1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ref2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ref3: Option<String>,

    // QR CS
    #[serde(rename = "csExtExpiryTime", skip_serializing_if = "Option::is_none")]
    cs_ext_expiry_time: Option<String>,
    #[serde(rename = "csNote", skip_serializing_if = "Option::is_none")]
    cs_note: Option<String>,
    #[serde(rename = "csUserDefined", skip_serializing_if = "Option::is_none")]
    cs_user_defined: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    invoice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    merchant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    terminal_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QrCodeResponse {
    status: Status,
    data: Option<QrCodeData>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    code: u64,
    description: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QrCodeData {
    #[serde(rename = "qrRawData")]
    qr_raw_data: String,

    #[serde(rename = "qrImage")]
    qr_image: String,

    #[serde(rename = "csExtExpiryTime")]
    expiry_time: Option<String>,

    #[serde(rename = "responseCode")]
    response_code: Option<String>,

    #[serde(rename = "qrCodeType")]
    qr_code_type: Option<String>,

    #[serde(rename = "qrcodeId")]
    qr_code_id: Option<String>,
}

pub async fn create_qrcode(
    data: Json<QrCodeRequest>,
    context: Data<FastJobContext>,
    _local_user_view: LocalUserView,
) -> FastJobResult<HttpResponse> {
    let scb = &context.settings().scb;
    let url = format!("{}v1/payment/qrcode/create", scb.url);

    let request_uid = Uuid::new_v4().to_string();

    let mut body = data.body.clone();
    body.merchant_id = Some(scb.merchant_id.clone());
    body.terminal_id = Some(scb.terminal_id.clone());

    let client = Client::new();
    let res = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("authorization", format!("Bearer {}", data.token))
        .header("resourceOwnerId", &scb.api_key)
        .header("requestUId", request_uid)
        .header("accept-language", "EN")
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<QrCodeResponse>()
        .await?;

    Ok(HttpResponse::Ok().json(res))
}
