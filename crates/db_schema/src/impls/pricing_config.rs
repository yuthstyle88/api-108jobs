use crate::{
  newtypes::{CurrencyId, PricingConfigId},
  source::pricing_config::{PricingConfig, PricingConfigInsertForm, PricingConfigUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};

use diesel::dsl::{insert_into, update};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Crud for PricingConfig {
  type InsertForm = PricingConfigInsertForm;
  type UpdateForm = PricingConfigUpdateForm;
  type IdType = PricingConfigId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(app_108jobs_db_schema_file::schema::pricing_config::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreatePricingConfig)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    config_id: PricingConfigId,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    update(app_108jobs_db_schema_file::schema::pricing_config::table.find(config_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdatePricingConfig)
  }
}

impl PricingConfig {
  pub async fn list_all(pool: &mut DbPool<'_>) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    app_108jobs_db_schema_file::schema::pricing_config::table
      .order(app_108jobs_db_schema_file::schema::pricing_config::is_active.desc())
      .then_order_by(app_108jobs_db_schema_file::schema::pricing_config::currency_id.asc())
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  pub async fn list_by_currency(
    pool: &mut DbPool<'_>,
    currency_id: CurrencyId,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    app_108jobs_db_schema_file::schema::pricing_config::table
      .filter(app_108jobs_db_schema_file::schema::pricing_config::currency_id.eq(currency_id))
      .filter(app_108jobs_db_schema_file::schema::pricing_config::is_active.eq(true))
      .order(app_108jobs_db_schema_file::schema::pricing_config::name.asc())
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  pub async fn get_active_for_currency(
    pool: &mut DbPool<'_>,
    currency_id: CurrencyId,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;

    let result = app_108jobs_db_schema_file::schema::pricing_config::table
      .filter(app_108jobs_db_schema_file::schema::pricing_config::currency_id.eq(currency_id))
      .filter(app_108jobs_db_schema_file::schema::pricing_config::is_active.eq(true))
      .first::<Self>(conn)
      .await
      .optional()
      .map_err(|_| FastJobErrorType::DatabaseError)?;

    Ok(result)
  }
}
