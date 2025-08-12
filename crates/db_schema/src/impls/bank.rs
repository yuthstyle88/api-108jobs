use diesel::ExpressionMethods;
#[cfg(feature = "full")]
use crate::{
  newtypes::BankId,
  source::bank::{Bank, BankInsertForm, BankUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};

#[cfg(feature = "full")]
use diesel::QueryDsl;
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::banks;
#[cfg(feature = "full")]
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use crate::source::bank::BanksResponse;

#[cfg(feature = "full")]
impl Crud for Bank {
  type InsertForm = BankInsertForm;
  type UpdateForm = BankUpdateForm;
  type IdType = BankId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(banks::table)
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
    diesel::update(banks::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }
}
impl Bank {
  pub async fn query_with_order_by(
    pool: &mut DbPool<'_>,
    order_by: Option<String>,
  ) -> FastJobResult<BanksResponse> {
    let conn = &mut get_conn(pool).await?;

    let mut query = banks::table.into_boxed();

    match order_by.as_deref() {
      Some("bank_name_asc") => {
        query = query.order(banks::name.asc());
      }
      Some("bank_name_desc") => {
        query = query.order(banks::name.desc());
      }
      _ => {
        // Default: newest first
        query = query.order(banks::created_at.desc());
      }
    }

    let items: Vec<Bank> = query.load(conn).await?;
    Ok(BanksResponse { banks: items })
  }
}
