use crate::newtypes::CurrencyId;
use crate::source::currency_rate_history::{CurrencyRateHistory, CurrencyRateHistoryInsertForm};
use crate::traits::Crud;
use crate::utils::{get_conn, DbPool};
use diesel::dsl::insert_into;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Crud for CurrencyRateHistory {
  type InsertForm = CurrencyRateHistoryInsertForm;
  type UpdateForm = (); // Not updatable
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(app_108jobs_db_schema_file::schema::currency_rate_history::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  async fn update(
    _pool: &mut DbPool<'_>,
    _id: Self::IdType,
    _form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    // Currency rate history is immutable
    Err(FastJobErrorType::DatabaseError.into())
  }
}

impl CurrencyRateHistory {
  pub async fn list_by_currency(
    pool: &mut DbPool<'_>,
    currency_id: CurrencyId,
    limit: Option<i64>,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    let mut query = app_108jobs_db_schema_file::schema::currency_rate_history::table
      .filter(app_108jobs_db_schema_file::schema::currency_rate_history::currency_id.eq(currency_id))
      .order(app_108jobs_db_schema_file::schema::currency_rate_history::changed_at.desc())
      .into_boxed();

    if let Some(limit) = limit {
      query = query.limit(limit);
    }

    query
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  pub async fn list_recent(
    pool: &mut DbPool<'_>,
    limit: i64,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    app_108jobs_db_schema_file::schema::currency_rate_history::table
      .order(app_108jobs_db_schema_file::schema::currency_rate_history::changed_at.desc())
      .limit(limit)
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }
}
