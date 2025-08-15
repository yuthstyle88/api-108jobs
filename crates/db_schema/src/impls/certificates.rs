use crate::source::certificates::{CertificateView};
use crate::{
  newtypes::{CertificateId, PersonId},
  source::certificates::{Certificates, CertificatesInsertForm, CertificatesUpdateForm, CertificateResponse},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, BoolExpressionMethods};
use diesel::dsl::{insert_into, not};
use diesel::upsert::excluded;
use diesel_async::RunQueryDsl;
use diesel_async::scoped_futures::ScopedFutureExt;
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
        .with_fastjob_type(FastJobErrorType::Deleted)
  }

  pub async fn save_certificate_list(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    certificates: &[CertificatesItem],
  ) -> FastJobResult<Vec<CertificateResponse>> {
    let conn = &mut get_conn(pool).await?;
    conn.build_transaction().run(|conn| {
      async move {
        let entries: Vec<(CertificatesInsertForm, String)> = certificates
        .iter()
        .filter_map(|i| i.name.as_ref()
        .map(|name| (CertificatesInsertForm::new(person_id, name.clone(), i.achieved_date.clone(), i.expires_date.clone(), i.url.clone()), name.clone())))
        .collect();
        let (forms, names_to_keep): (Vec<_>, Vec<_>) = entries.into_iter().unzip();

        if forms.is_empty() {
          diesel::delete(certificates::table.filter(certificates::person_id.eq(person_id)))
          .execute(conn).await
          .with_fastjob_type(FastJobErrorType::DatabaseError)?;
          return Ok(Vec::new());
        }

        let upserted = insert_into(certificates::table)
        .values(&forms)
        .on_conflict((certificates::person_id, certificates::name))
        .do_update()
        .set((
          certificates::achieved_date.eq(excluded(certificates::achieved_date)),
          certificates::expires_date.eq(excluded(certificates::expires_date)),
          certificates::url.eq(excluded(certificates::url)),
          certificates::updated_at.eq(Utc::now()),
        ))
        .returning(certificates::all_columns)
        .get_results::<Certificates>(conn).await
        .with_fastjob_type(FastJobErrorType::DatabaseError)?;

        diesel::delete(
          certificates::table.filter(
            certificates::person_id.eq(person_id)
            .and(not(certificates::name.eq_any(&names_to_keep))),
          ),
        )
        .execute(conn).await
        .with_fastjob_type(FastJobErrorType::DatabaseError)?;

        Ok(upserted.into_iter().map(|cert| CertificateResponse {
            id: cert.id,
            person_id: cert.person_id,
            name: cert.name,
            achieved_date: cert.achieved_date,
            expires_date: cert.expires_date,
            url: cert.url,
            created_at: cert.created_at,
            updated_at: cert.updated_at,
        }).collect())
      }.scope_boxed()
    }).await
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

