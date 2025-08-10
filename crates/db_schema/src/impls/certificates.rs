use crate::source::certificates::{CertificateView, UpdateCertificateRequestItem};
use crate::{
  newtypes::{CertificateId, PersonId},
  source::certificates::{Certificates, CertificatesInsertForm, CertificatesUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::certificates;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

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
}
impl From<Certificates> for CertificateView {
  fn from(parts: Certificates) -> Self {
    CertificateView {
      id: parts.id,
      name:parts.name,
      achieved_date: parts.achieved_date,
      expires_date: parts.expires_date,
      url: parts.url.unwrap(),
    }
  }
}
impl From<UpdateCertificateRequestItem> for CertificatesUpdateForm {

  fn from(data: UpdateCertificateRequestItem) -> Self {
    Self{
      name: Some(data.name),
      achieved_date: data.achieved_date,
      expires_date: data.expires_date,
      url: Some(data.url),
    }
  }
}
