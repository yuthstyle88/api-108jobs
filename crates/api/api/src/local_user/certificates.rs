use actix_web::web::{Data, Json};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::certificates::{CertificateView, Certificates, CertificatesRequest, CertificateResponse};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResponse {
  pub success: bool,
  pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificatesListResponse {
    pub certificates: Vec<CertificateResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificatesViewResponse {
    pub certificates: Vec<CertificateView>,
}

pub async fn save_certificates(
    data: Json<CertificatesRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<CertificatesListResponse>> {
    let person_id = local_user_view.person.id;

    let certificate_responses = Certificates::save_certificate_list(
        &mut context.pool(),
        person_id,
        &data.certificates,
    ).await?;

    Ok(Json(CertificatesListResponse {
        certificates: certificate_responses,
    }))
}

pub async fn list_certificates(
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<CertificatesViewResponse>> {
    let person_id = Some(local_user_view.person.id);
    let certificates = Certificates::query_with_filters(&mut context.pool(), person_id).await.unwrap_or_else(|_| Vec::new());
    Ok(Json(CertificatesViewResponse {
        certificates,
    }))
}
