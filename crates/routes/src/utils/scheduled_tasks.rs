use actix_web::web::Data;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_core::error::FastJobResult;
use app_108jobs_db::{
  enums::TopUpStatus,
  schema::{
    captcha_answer,
    top_up_requests::{cs_ext_expiry_time, dsl::top_up_requests, id, status},
  },
  utils::{get_conn, now, DbPool},
};
use chrono::Utc;
use clokwerk::{AsyncScheduler, TimeUnits as CTimeUnits};
use diesel::{dsl::IntervalDsl, BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use std::time::Duration;
use tracing::{info, warn};

/// Schedules various cleanup tasks for app_108jobs in a background thread
pub async fn setup(context: Data<FastJobContext>) -> FastJobResult<()> {
  // https://github.com/mdsherry/clokwerk/issues/38
  let mut scheduler = AsyncScheduler::with_tz(Utc);

  let context_1 = context.clone();
  // Check expired wallet topups every 10 minutes
  scheduler.every(CTimeUnits::minutes(30)).run(move || {
    let context = context_1.clone();

    async move {
      update_expired_wallet_topups(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to update expired wallet topups: {e}"))
        .ok();
    }
  });

  // Manually run the scheduler in an event loop
  loop {
    scheduler.run_pending().await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
  }
}

async fn _delete_expired_captcha_answers(pool: &mut DbPool<'_>) -> FastJobResult<()> {
  let mut conn = get_conn(pool).await?;

  diesel::delete(
    captcha_answer::table.filter(captcha_answer::published_at.lt(now() - IntervalDsl::minutes(10))),
  )
  .execute(&mut conn)
  .await?;
  info!("Done.");

  Ok(())
}

async fn update_expired_wallet_topups(pool: &mut DbPool<'_>) -> FastJobResult<()> {
  let mut conn = get_conn(pool).await?;
  let now_utc = Utc::now();
  let expired_ids: Vec<i32> = top_up_requests
    .filter(
      cs_ext_expiry_time
        .lt(now_utc)
        .and(status.ne(TopUpStatus::Expired))
        .and(status.ne(TopUpStatus::Success)),
    )
    .select(id)
    .load(&mut conn)
    .await?;

  info!(
    "Found {} expired topup(s): {:?}",
    expired_ids.len(),
    expired_ids
  );

  if !expired_ids.is_empty() {
    let updated = diesel::update(top_up_requests.filter(id.eq_any(&expired_ids)))
      .set(status.eq(TopUpStatus::Expired))
      .execute(&mut conn)
      .await?;

    info!("Marked {} topup(s) as Expired", updated);
  }

  Ok(())
}
