use actix_web::web::{Data, Json};
use chrono::NaiveDate;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateIdentityCard {
  pub address_id: Option<String>,
  pub id_number: String,
  pub issued_date: String,
  pub expiry_date: String,
  pub full_name: String,
  pub date_of_birth: String,
  pub nationality: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIdentityCard {
  pub address_id: Option<String>,
  pub id_number: Option<String>,
  pub issued_date: Option<String>,
  pub expiry_date: Option<String>,
  pub full_name: Option<String>,
  pub date_of_birth: Option<String>,
  pub nationality: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdentityCardOperationResponse {
  pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdentityCardListResponse {
  pub identity_cards: Vec<IdentityCardDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdentityCardDto {
  pub id: Option<String>,
  pub address_id: Option<String>,
  pub id_number: String,
  pub issued_date: String,
  pub expiry_date: String,
  pub full_name: String,
  pub date_of_birth: String,
  pub nationality: String,
  pub is_verified: bool,
  pub created_at: String,
}

fn is_all_digits(s: &str) -> bool { s.chars().all(|c| c.is_ascii_digit()) }

fn parse_date(s: &str) -> Result<NaiveDate, FastJobErrorType> {
  NaiveDate::parse_from_str(s, "%Y-%m-%d")
    .map_err(|_| FastJobErrorType::InvalidField("Invalid date format, expected YYYY-MM-DD".into()))
}

pub async fn create_identity_card(
  data: Json<CreateIdentityCard>,
  _context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<IdentityCardOperationResponse>> {
  // Ensure user is present
  let _user_id = local_user_view.local_user.id;

  let payload = data.into_inner();

  if payload.id_number.trim().is_empty() {
    return Err(FastJobErrorType::EmptyIDNumber.into());
  }
  if !(payload.id_number.len() == 13 && is_all_digits(&payload.id_number)) {
    return Err(FastJobErrorType::InvalidIDNumber.into());
  }
  if payload.full_name.trim().is_empty() {
    return Err(FastJobErrorType::EmpltyFullName.into());
  }
  if payload.nationality.trim().is_empty() {
    return Err(FastJobErrorType::EmptyNationality.into());
  }

  let issued = parse_date(&payload.issued_date)?;
  let expiry = parse_date(&payload.expiry_date)?;
  if expiry <= issued {
    return Err(FastJobErrorType::InvalidIssueAndExpire.into());
  }

  // NOTE: Persistence not implemented yet; this endpoint performs validation only.
  Ok(Json(IdentityCardOperationResponse { success: true }))
}

pub async fn update_identity_card(
  data: Json<UpdateIdentityCard>,
  _context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<IdentityCardOperationResponse>> {
  let _user_id = local_user_view.local_user.id;
  let p = data.into_inner();

  if let Some(ref id_number) = p.id_number {
    if id_number.trim().is_empty() {
      return Err(FastJobErrorType::EmptyIDNumber.into());
    }
    if !(id_number.len() == 13 && is_all_digits(id_number)) {
      return Err(FastJobErrorType::InvalidIDNumber.into());
    }
  }
  if let Some(ref full_name) = p.full_name {
    if full_name.trim().is_empty() {
      return Err(FastJobErrorType::EmpltyFullName.into());
    }
  }
  if let Some(ref nationality) = p.nationality {
    if nationality.trim().is_empty() {
      return Err(FastJobErrorType::EmptyNationality.into());
    }
  }
  if let (Some(issued), Some(expiry)) = (&p.issued_date, &p.expiry_date) {
    let issued = parse_date(issued)?;
    let expiry = parse_date(expiry)?;
    if expiry <= issued {
      return Err(FastJobErrorType::InvalidIssueAndExpire.into());
    }
  }

  Ok(Json(IdentityCardOperationResponse { success: true }))
}
