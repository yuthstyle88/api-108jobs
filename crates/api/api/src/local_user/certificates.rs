use actix_web::web::{Data, Json};
use lemmy_api_common::account::{CertificatesRequest, UpdateCertificateRequest, DeleteItemRequest};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::certificates::{Certificates, CertificatesInsertForm, CertificatesUpdateForm};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;
use chrono::NaiveDate;
use lemmy_db_schema::newtypes::CertificateId;

// Helper function to parse date strings
fn parse_date_string(date_str: &Option<String>) -> Option<NaiveDate> {
    date_str.as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
}

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
                Certificates::update_by_id_and_person(
                    &mut context.pool(), 
                    id, 
                    person_id, 
                    cert.name.clone(),
                    parse_date_string(&cert.achieved_date),
                    parse_date_string(&cert.expires_date),
                    cert.url.clone(),
                ).await?
            }
            // Create new certificate record
            None => {
                let form = CertificatesInsertForm::new(
                    person_id,
                    cert.name.clone(),
                    parse_date_string(&cert.achieved_date),
                    parse_date_string(&cert.expires_date),
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
) -> FastJobResult<Json<Vec<Certificates>>> {
    let person_id = local_user_view.person.id;
    let certificates = Certificates::read_by_person_id(&mut context.pool(), person_id).await?;
    Ok(Json(certificates))
}

pub async fn delete_certificates(
    data: Json<Vec<i32>>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<String>> {
    let person_id = local_user_view.person.id;
    
    // Delete specific certificates records
    for certificate_id in data.iter() {
        Certificates::delete_by_id_and_person(&mut context.pool(), *certificate_id, person_id).await?;
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