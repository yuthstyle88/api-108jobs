use crate::{
  newtypes::CurrencyId,
  source::currency::{Currency, CurrencyInsertForm, CurrencyUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};

use diesel::dsl::{insert_into, update};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Crud for Currency {
  type InsertForm = CurrencyInsertForm;
  type UpdateForm = CurrencyUpdateForm;
  type IdType = CurrencyId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(app_108jobs_db_schema_file::schema::currency::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateCurrency)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    currency_id: CurrencyId,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    update(app_108jobs_db_schema_file::schema::currency::table.find(currency_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateCurrency)
  }
}

impl Currency {
  pub async fn list_all(pool: &mut DbPool<'_>) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    app_108jobs_db_schema_file::schema::currency::table
      .order(app_108jobs_db_schema_file::schema::currency::is_active.desc())
      .then_order_by(app_108jobs_db_schema_file::schema::currency::code.asc())
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  pub async fn get_default(pool: &mut DbPool<'_>) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;

    let result = app_108jobs_db_schema_file::schema::currency::table
      .filter(app_108jobs_db_schema_file::schema::currency::is_default.eq(true))
      .first::<Self>(conn)
      .await
      .optional()
      .map_err(|_| FastJobErrorType::DatabaseError)?;

    Ok(result)
  }

  pub async fn get_by_code(pool: &mut DbPool<'_>, code: &str) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;

    let result = app_108jobs_db_schema_file::schema::currency::table
      .filter(app_108jobs_db_schema_file::schema::currency::code.eq(code))
      .first::<Self>(conn)
      .await
      .optional()
      .map_err(|_| FastJobErrorType::DatabaseError)?;

    Ok(result)
  }

  /// Find currency by ISO 4217 numeric currency code
  /// Used for mapping payment gateway responses (SCB, etc.)
  /// Example: 764 = THB, 360 = IDR, 704 = VND
  pub async fn get_by_numeric_code(pool: &mut DbPool<'_>, numeric_code: i32) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;

    let result = app_108jobs_db_schema_file::schema::currency::table
      .filter(app_108jobs_db_schema_file::schema::currency::numeric_code.eq(numeric_code))
      .first::<Self>(conn)
      .await
      .optional()
      .map_err(|_| FastJobErrorType::DatabaseError)?;

    Ok(result)
  }
}
