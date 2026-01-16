use crate::payments::get_token::fetch_scb_token;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use chrono::{Duration, Utc};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::source::top_up_request::{TopUpRequest, TopUpRequestInsertForm};
use app_108jobs_db_schema::traits::Crud;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_utils::error::FastJobResult;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QrCodeRequest {
  body: QrCodeBody,
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
  expiry_time: String,

  #[serde(rename = "responseCode")]
  response_code: Option<String>,

  #[serde(rename = "qrCodeType")]
  qr_code_type: Option<String>,

  #[serde(rename = "qrcodeId")]
  qr_code_id: String,

  #[serde(rename = "amount")]
  amount: String,

  #[serde(rename = "currencyName")]
  currency_name: String,
}

pub async fn create_qrcode(
  data: Json<QrCodeRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<HttpResponse> {
  let scb = &context.settings().scb;
  let url = format!("{}v1/payment/qrcode/create", scb.url);

  // Step 1: Get SCB token
  let token = fetch_scb_token(&context).await?;

  // Step 2: Prepare body
  let mut data = data.into_inner();
  data.body.merchant_id = Some(scb.merchant_id.clone());
  data.body.terminal_id = Some(scb.terminal_id.clone());

  // Step 3: Make request
  let request_uid = Uuid::new_v4().to_string();
  let client = Client::new();

  let res = client
    .post(&url)
    .header("Content-Type", "application/json")
    .header("authorization", format!("Bearer {}", token))
    .header("resourceOwnerId", &scb.api_key)
    .header("requestUId", request_uid)
    .header("accept-language", "EN")
    .json(&data.body)
    .send()
    .await?
    .error_for_status()?
    .json::<QrCodeResponse>()
    .await?;

  if let Some(ref data) = res.data {
    let expiry_time = Utc::now() + Duration::minutes(1);

    let insert_form = TopUpRequestInsertForm {
      local_user_id: local_user_view.local_user.id,
      amount: data.amount.parse().unwrap_or(0.0),
      currency_name: data.currency_name.clone(),
      qr_id: data.qr_code_id.clone(),
      cs_ext_expiry_time: expiry_time,
      paid_at: None,
    };

    let _created = TopUpRequest::create(&mut context.pool(), &insert_form).await?;
  }

  Ok(HttpResponse::Ok().json(res))
}
