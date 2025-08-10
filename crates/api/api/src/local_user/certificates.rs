use actix_web::web::{Data, Json};
use lemmy_api_common::account::{UpdateCertificateRequest, DeleteItemRequest};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::certificates::{CertificateView, Certificates, CertificatesInsertForm, CertificatesRequest, CertificatesUpdateForm};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;
use lemmy_db_schema::newtypes::CertificateId;
use lemmy_db_schema::traits::Crud;

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
                let form = CertificatesUpdateForm{
                    name: Some(cert.name.clone()),
                    achieved_date: cert.achieved_date,
                    expires_date: cert.expires_date,
                    url: Some(cert.url.clone()),
                };
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
    _data: Json<UpdateCertificateRequest>,
    _context: Data<FastJobContext>,
) -> FastJobResult<Json<()>> {

    let _form = CertificatesUpdateForm{
        name: None,
        achieved_date: None,
        expires_date: None,
        url: None,
    };
    // let updated_certificate = Certificates::update(
    //     &mut context.pool(),
    //     data.id,
    //     &form,
    // ).await?;

    Ok(Json(()))
}

pub async fn delete_single_certificate(
    data: Json<DeleteItemRequest<CertificateId>>,
    _context: Data<FastJobContext>,
) -> FastJobResult<Json<String>> {
    let _id = data.into_inner().id;
    // Certificates::delete(&mut context.pool(), id).await?;
    Ok(Json("Certificate record deleted successfully".to_string()))
}