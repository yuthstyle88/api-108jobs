use crate::{
  newtypes::{PostId, RiderId},
  source::delivery_location_history::{DeliveryLocationHistory, DeliveryLocationHistoryInsertForm},
  utils::{get_conn, DbPool},
};

use diesel::dsl::insert_into;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;

use app_108jobs_db_schema_file::schema::delivery_location_history;
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl DeliveryLocationHistory {
  pub async fn create(
    pool: &mut DbPool<'_>,
    form: &DeliveryLocationHistoryInsertForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(delivery_location_history::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateDeliveryLocationHistory)
  }

  pub async fn list_for_post(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    limit: i64,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    delivery_location_history::table
      .filter(delivery_location_history::post_id.eq(post_id))
      .order(delivery_location_history::recorded_at.desc())
      .limit(limit)
      .select(Self::as_select())
      .load(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn list_for_rider(
    pool: &mut DbPool<'_>,
    rider_id: RiderId,
    limit: i64,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    delivery_location_history::table
      .filter(delivery_location_history::rider_id.eq(rider_id))
      .order(delivery_location_history::recorded_at.desc())
      .limit(limit)
      .select(Self::as_select())
      .load(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}
