use actix_web::web::Data;
use chrono::Utc;
use clokwerk::{AsyncScheduler, TimeUnits as CTimeUnits};
use diesel::{dsl::IntervalDsl, sql_query, BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::utils::{get_conn, now, DbPool};
use app_108jobs_db_schema_file::enums::TopUpStatus;
use app_108jobs_db_schema_file::schema::captcha_answer;
use app_108jobs_db_schema_file::schema::top_up_requests::dsl::top_up_requests;
use app_108jobs_db_schema_file::schema::top_up_requests::{cs_ext_expiry_time, id, status};
use app_108jobs_utils::error::FastJobResult;
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

/// Re-calculate the site and community active counts every 12 hours
async fn _active_counts(pool: &mut DbPool<'_>) -> FastJobResult<()> {
  info!("Updating active site and community aggregates ...");

  let mut conn = get_conn(pool).await?;

  let intervals = vec![
    ("1 day", "day"),
    ("1 week", "week"),
    ("1 month", "month"),
    ("6 months", "half_year"),
  ];

  for (full_form, abbr) in &intervals {
    let update_site_stmt = format!(
      "update local_site set users_active_{} = (select r.site_aggregates_activity('{}')) where site_id = 1",
      abbr, full_form
    );
    sql_query(update_site_stmt).execute(&mut conn).await?;

    let update_community_stmt = format!("update community ca set users_active_{} = mv.count_ from r.community_aggregates_activity('{}') mv where ca.id = mv.community_id_", abbr, full_form);
    sql_query(update_community_stmt).execute(&mut conn).await?;
  }

  let update_interactions_stmt = "update community ca set interactions_month = mv.count_ from r.community_aggregates_interactions('1 month') mv where ca.id = mv.community_id_";
  sql_query(update_interactions_stmt)
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
