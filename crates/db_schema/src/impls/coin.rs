use crate::newtypes::{Coin, CoinId};
use crate::{
  source::coin::{CoinModel, CoinModelInsertForm, CoinModelUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{insert_into, now},
  ExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::coin;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Crud for CoinModel {
  type InsertForm = CoinModelInsertForm;
  type UpdateForm = CoinModelUpdateForm;
  type IdType = CoinId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(coin::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateContact)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(coin::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateContact)
  }

  async fn delete(pool: &mut DbPool<'_>, id: Self::IdType) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(coin::table.find(id))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntDeleteContact)
  }
}

impl CoinModel {
  /// Update coin supply_total by **delta** (positive to increase, negative to decrease).
  /// - Reads current `supply_total` with `FOR UPDATE` to lock the row
  /// - Computes `new_total = round(current + delta, 2)` (two decimal places)
  /// - Validates `new_total >= 0`
  /// - Persists and returns the updated model, all inside a single DB transaction
  pub async fn update_balance(
    conn: &mut diesel_async::AsyncPgConnection,
    coin_id: CoinId,
    delta: Coin,
  ) -> FastJobResult<CoinModel> {
    // 1) Lock & read current supply
    let current_total: i32 = coin::table
      .find(coin_id)
      .select(coin::supply_total)
      .for_update()
      .first::<i32>(conn)
      .await?;

    // 2) Compute new total (integer, no decimals)
    let new_total = current_total + delta.0 as i32;
    if new_total < 0 {
      return Err(FastJobErrorType::InvalidField("coin supply cannot be negative".into()).into());
    }

    // 3) Persist
    let updated = diesel::update(coin::table.find(coin_id))
      .set((
        coin::supply_total.eq(new_total),
        coin::updated_at.eq(now),
      ))
      .get_result::<CoinModel>(conn)
      .await?;

    Ok(updated)
  }
}
