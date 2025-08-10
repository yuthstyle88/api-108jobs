use actix_web::web::{Data, Json};
use lemmy_api_common::account::DeleteItemRequest;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::CertificateId;
use lemmy_db_schema::source::certificates::{CertificateView, Certificates, CertificatesInsertForm, CertificatesRequest, CertificatesUpdateForm, UpdateCertificateRequestItem};
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub async fn save_certificates(
    data: Json<CertificatesRequest>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<Vec<Certificates>>> {
    let person_id = local_user_view.person.id;

    let mut saved_certificates = Vec::new();
    for cert in &data.certificates {
        let saved = match cert.id {
            // Update existing certificate record
            Some(id) => {
                let form = cert.into();
                Certificates::update(
                    &mut context.pool(), 
                    id, 
                    &form,
                ).await?
            }
            // Create new certificate record
            None => {
                let form = CertificatesInsertForm::new(
                    person_id,
                    cert.name.clone(),
                    cert.achieved_date,
                    cert.expires_date,
                    cert.url.clone(),
                );
                Certificates::create(&mut context.pool(), &form).await?
            }
        };
        saved_certificates.push(saved);
    }

    Ok(Json(saved_certificates))
}

pub async fn list_certificates(
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<Vec<CertificateView>>> {
    let person_id = Some(local_user_view.person.id);
    let certificates = Certificates::query_with_filters(&mut context.pool(), person_id).await?;
    Ok(Json(certificates))
}

pub async fn delete_certificates(
    data: Json<Vec<CertificateId>>,
    context: Data<FastJobContext>,
) -> FastJobResult<Json<String>> {
    // Delete specific certificates records
    for certificate_id in data.iter() {
        Certificates::delete(&mut context.pool(), *certificate_id).await?;
    }

    Ok(Json("Certificates records deleted successfully".to_string()))
}

pub async fn update_certificate(
    data: Json<UpdateCertificateRequestItem>,
    context: Data<FastJobContext>,
) -> FastJobResult<Json<()>> {
    // Extract request once to avoid use-after-move, then take id before conversion
    let req = data.into_inner();
    let id = req.id;
    let form: CertificatesUpdateForm = req.into();
    // Apply update
    let _updated = Certificates::update(&mut context.pool(), id, &form).await?;

    Ok(Json(()))
}

pub async fn delete_single_certificate(
    data: Json<DeleteItemRequest<CertificateId>>,
    context: Data<FastJobContext>,
) -> FastJobResult<Json<String>> {
    let id = data.into_inner().id;
    Certificates::delete(&mut context.pool(), id).await?;
    Ok(Json("Certificate record deleted successfully".to_string()))
}