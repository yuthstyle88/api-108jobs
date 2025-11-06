use crate::payments::get_token::fetch_scb_token;
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use chrono::Utc;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::wallet_topup::{WalletTopup, WalletTopupUpdateForm};
use lemmy_db_schema_file::enums::TopupStatus;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QrInquiryResponse {
  pub status: Status,
  pub data: Option<QrInquiryData>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
  pub code: i32,
  pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QrInquiryData {
  pub transaction_id: Option<String>,
  pub amount: Option<String>,
  pub transaction_dateand_time: String,
  #[serde(rename = "merchantPAN")]
  pub merchant_pan: Option<String>,
  #[serde(rename = "consumerPAN")]
  pub consumer_pan: Option<String>,
  pub currency_code: Option<String>,
  pub merchant_id: Option<String>,
  pub terminal_id: Option<String>,
  pub qr_id: String,
  pub trace_no: Option<String>,
  pub authorize_code: Option<String>,
  pub payment_method: Option<String>,
  pub transaction_type: Option<String>,
  pub channel_code: Option<String>,
  pub invoice: Option<String>,
  pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QrInquiryRequest {
  pub qr_id: String,
}

pub async fn inquire_qrcode(
  data: Json<QrInquiryRequest>,
  context: Data<FastJobContext>,
  _local_user_view: LocalUserView,
) -> FastJobResult<HttpResponse> {
  let scb = context.settings().scb.clone();
  let url = format!("{}v1/payment/qrcode/creditcard/{}", scb.url, data.qr_id);

  let token = fetch_scb_token(&context).await?;

  let request_uid = Uuid::new_v4().to_string();

  let client = Client::new();
  let resp = client
    .get(url)
    .header("authorization", format!("Bearer {}", token))
    .header("resourceOwnerId", scb.api_key.clone())
    .header("requestUId", request_uid)
    .header("accept-language", "EN")
    .send()
    .await?;

  let text = resp.text().await?;

  if let Ok(parsed) = serde_json::from_str::<QrInquiryResponse>(&text) {
    if parsed.status.code == 2104 {
      return Err(FastJobErrorType::StillDoNotPayYet.into());
    }

    if let Some(ref data) = parsed.data {
      let wallet_topup_update_form = WalletTopupUpdateForm {
        status: Some(TopupStatus::Success),
        updated_at: Some(Utc::now()),
        paid_at: Some(Some(data.transaction_dateand_time.parse()?)),
        transferred: None,
      };
      let _updated = WalletTopup::update_by_qr_id(
        &mut context.pool(),
        data.qr_id.clone(),
        &wallet_topup_update_form,
      )
      .await?;
    }

    return Ok(HttpResponse::Ok().json(parsed));
  }

  Err(FastJobErrorType::ReturnedNonJSONResponse.into())
}
