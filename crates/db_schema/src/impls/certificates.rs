use crate::source::certificates::{CertificateView};
use crate::{
  newtypes::{CertificateId, PersonId},
  source::certificates::{Certificates, CertificatesInsertForm, CertificatesUpdateForm, CertificateResponse},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{certificates};
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use crate::source::certificates::{CertificatesItem};

impl Crud for Certificates {
  type InsertForm = CertificatesInsertForm;
  type UpdateForm = CertificatesUpdateForm;
  type IdType = CertificateId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(certificates::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(certificates::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }
}
impl Certificates {
  pub async fn query_with_filters(
    pool: &mut DbPool<'_>,
    person_id: Option<PersonId>,
  ) -> FastJobResult<Vec<CertificateView>> {
    let conn = &mut get_conn(pool).await?;

    let mut query = certificates::table
    .into_boxed();

    if let Some(id) = person_id {
      query = query.filter(certificates::person_id.eq(id));
    }

    let items: Vec<Certificates> = query
    .load(conn)
    .await?;

    if items.is_empty() {
      return Err(FastJobErrorType::NotFound.into());
    }

    Ok(items.into_iter().map(Into::into).collect::<Vec<CertificateView>>())
  }

  pub async fn delete_not_in_list(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    certificate_ids: &[CertificateId],
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;

    diesel::delete(certificates::table)
        .filter(certificates::person_id.eq(person_id))
        .filter(certificates::id.ne_all(certificate_ids))
        .execute(conn)
        .await
        .with_fastjob_type(FastJobErrorType::CouldntDeleteCertificate)
  }

  pub async fn save_certificate_list(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    certificates: &[CertificatesItem],
  ) -> FastJobResult<Vec<CertificateResponse>> {
    let conn = &mut get_conn(pool).await?;

    conn.build_transaction()
        .run(|conn| {
          Box::pin(async move {
            let mut saved_certificates = Vec::new();
            let mut certificate_ids = Vec::new();

            for certificate_item in certificates {
              match certificate_item.id {
                Some(id) => {
                  // Update existing certificate
                  let form = CertificatesUpdateForm {
                    name: certificate_item.name.clone(),
                    achieved_date: certificate_item.achieved_date.clone(),
                    expires_date: certificate_item.expires_date.clone(),
                    url: Some(certificate_item.url.clone()),
                  };
                  let updated = Self::update(&mut conn.into(), id, &form).await?;
                  certificate_ids.push(id);
                  saved_certificates.push(updated);
                }
                None => {
                  // Create new certificate
                  if let (Some(name), Some(achieved_date), Some(expires_date), Some(url)) = (&certificate_item.name, &certificate_item.achieved_date, &certificate_item.expires_date, &certificate_item.url) {
                    let form = CertificatesInsertForm::new(
                      person_id,
                      name.clone(),
                      Some(achieved_date.clone()),
                      Some(expires_date.clone()),
                      Some(url.clone())
                    );
                    let created = Self::create(&mut conn.into(), &form).await?;
                    certificate_ids.push(created.id);
                    saved_certificates.push(created);
                  }
                }
              }
            }

            // Delete any records not in the current list
            Self::delete_not_in_list(&mut conn.into(), person_id, &certificate_ids).await?;

            // Convert to response format
            let certificate_responses: Vec<CertificateResponse> = saved_certificates
                .into_iter()
                .map(|cert| CertificateResponse {
                    id: cert.id,
                    person_id: cert.person_id,
                    name: cert.name,
                    achieved_date: cert.achieved_date,
                    expires_date: cert.expires_date,
                    url: cert.url,
                    created_at: cert.created_at,
                    updated_at: cert.updated_at,
                })
                .collect();

            Ok(certificate_responses)
          }) as _
        })
        .await
  }
}
impl From<Certificates> for CertificateView {
  fn from(parts: Certificates) -> Self {
    CertificateView {
      id: parts.id,
      name:parts.name,
      achieved_date: parts.achieved_date,
      expires_date: parts.expires_date,
      url: parts.url,
    }
  }
}

