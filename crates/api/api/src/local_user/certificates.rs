use actix_web::web::{Data, Json};
use lemmy_api_common::account::DeleteItemRequest;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::CertificateId;
use lemmy_db_schema::source::certificates::{CertificateView, Certificates, CertificatesInsertForm, CertificatesRequest, CertificatesUpdateForm, UpdateCertificateRequestItem, DeleteCertificatesRequest};
use lemmy_db_schema::traits::Crud;
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
    pub certificates: Vec<CertificateView>,
}

pub async fn save_certificates(
    data: Json<CertificatesRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<CertificatesListResponse>> {
    let person_id = local_user_view.person.id;

    let mut saved_certificates = Vec::new();
    for cert in data.certificates.clone() {
        match (cert.id, cert.deleted) {
            // Delete existing certificate
            (Some(id), true) => {
                Certificates::delete(&mut context.pool(), id).await?;
                // Don't add to saved_certificates since it's deleted
            }
            // Update existing certificate
            (Some(id), false) => {
                let form: CertificatesUpdateForm = cert.into();
                let updated = Certificates::update(&mut context.pool(), id, &form).await?;
                saved_certificates.push(updated);
            }
            // Create new certificate
            (None, false) => {
                if let Some(name) = cert.name {
                    let form = CertificatesInsertForm::new(
                        person_id,
                        name,
                        Some(cert.achieved_date), // achieved_date is now required in the request
                        cert.expires_date,
                        cert.url,
                    );
                    let created = Certificates::create(&mut context.pool(), &form).await?;
                    saved_certificates.push(created);
                }
            }
            // Invalid: trying to delete without ID
            (None, true) => {
                // Skip invalid delete requests
            }
        }
    }

    let certificate_views: Vec<CertificateView> = saved_certificates.into_iter().map(|cert| CertificateView::from(cert)).collect();
    Ok(Json(CertificatesListResponse {
        certificates: certificate_views,
    }))
}

pub async fn list_certificates(
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<CertificatesListResponse>> {
    let person_id = Some(local_user_view.person.id);
    let certificates = Certificates::query_with_filters(&mut context.pool(), person_id).await.unwrap_or_else(|_| Vec::new());
    Ok(Json(CertificatesListResponse {
        certificates,
    }))
}

pub async fn delete_certificates(
    data: Json<DeleteCertificatesRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<DeleteResponse>> {
    let person_id = local_user_view.person.id;
    let mut deleted_count = 0;

    for certificate_id in data.certificate_ids.clone() {
        // First verify the certificate belongs to the user
        if let Ok(certificate) = Certificates::read(&mut context.pool(), certificate_id).await {
            if certificate.person_id == person_id {
                Certificates::delete(&mut context.pool(), certificate_id).await?;
                deleted_count += 1;
            }
        }
    }

    Ok(Json(DeleteResponse {
        success: true,
        message: format!("{} records deleted successfully", deleted_count),
    }))
}

pub async fn update_certificate(
    data: Json<UpdateCertificateRequestItem>,
    context: Data<FastJobContext>,
) -> FastJobResult<Json<()>> {
    // Extract request once to avoid use-after-move, then take id before conversion
    let req = data.into_inner();
    if let Some(id )  = req.id {
        let form: CertificatesUpdateForm = req.into();
        // Apply update
        let _updated = Certificates::update(&mut context.pool(), id, &form).await?;
    }
    Ok(Json(()))
}

pub async fn delete_single_certificate(
    data: Json<DeleteItemRequest<CertificateId>>,
    context: Data<FastJobContext>,
) -> FastJobResult<Json<DeleteResponse>> {
    let id = data.into_inner().id;
    Certificates::delete(&mut context.pool(), id).await?;
    Ok(Json(DeleteResponse {
        success: true,
        message: "1 record deleted successfully".to_string(),
    }))
}