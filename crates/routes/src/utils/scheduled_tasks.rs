use actix_web::web::Data;
use chrono::{DateTime, TimeZone, Utc};
use clokwerk::{AsyncScheduler, TimeUnits as CTimeUnits};
use diesel::{
  dsl::{exists, not, IntervalDsl},
  query_builder::AsQuery,
  sql_query,
  sql_types::{Integer, Timestamptz},
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
  QueryableByName,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_api_utils::{
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_db_schema::{
  source::{
    community::Community,
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
  utils::{functions::coalesce, get_conn, now, uplete, DbPool},
};
use lemmy_db_schema_file::schema::{
  captcha_answer,
  community,
  community_actions,
  person,
  post,
};
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use std::time::Duration;
use tracing::{info, warn};

/// Schedules various cleanup tasks for lemmy in a background thread
pub async fn setup(context: Data<FastJobContext>) -> FastJobResult<()> {
  // https://github.com/mdsherry/clokwerk/issues/38
  let mut scheduler = AsyncScheduler::with_tz(Utc);

  let context_1 = context.clone();
  // Every 10 minutes update hot ranks, delete expired captchas and publish scheduled posts
  scheduler.every(CTimeUnits::minutes(10)).run(move || {
    let context = context_1.clone();

    async move {
      update_hot_ranks(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to update hot ranks: {e}"))
        .ok();
      delete_expired_captcha_answers(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to delete expired captcha answers: {e}"))
        .ok();
      publish_scheduled_posts(&context)
        .await
        .inspect_err(|e| warn!("Failed to publish scheduled posts: {e}"))
        .ok();
    }
  });

  let context_1 = context.clone();
  // Update active counts expired bans and unpublished posts every hour
  scheduler.every(CTimeUnits::hour(1)).run(move || {
    let context = context_1.clone();

    async move {
      active_counts(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to update active counts: {e}"))
        .ok();
      update_banned_when_expired(&mut context.pool())
        .await
        .inspect_err(|e| warn!("Failed to update expired bans: {e}"))
        .ok();
    }
  });

  // Manually run the scheduler in an event loop
  loop {
    scheduler.run_pending().await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
  }
}

/// Update the hot_rank columns for the aggregates tables
/// Runs in batches until all necessary rows are updated once
async fn update_hot_ranks(pool: &mut DbPool<'_>) -> FastJobResult<()> {
  info!("Updating hot ranks for all history...");

  let mut conn = get_conn(pool).await?;

  process_post_aggregates_ranks_in_batches(&mut conn).await?;

  process_ranks_in_batches(
    &mut conn,
    "comment",
    "a.hot_rank != 0",
    "SET hot_rank = r.hot_rank(a.score, a.published_at)",
  )
  .await?;

  process_ranks_in_batches(
    &mut conn,
    "community",
    "a.hot_rank != 0",
    "SET hot_rank = r.hot_rank(a.subscribers, a.published_at)",
  )
  .await?;

  info!("Finished hot ranks update!");
  Ok(())
}

#[derive(QueryableByName)]
struct HotRanksUpdateResult {
  #[diesel(sql_type = Timestamptz)]
  published_at: DateTime<Utc>,
}

/// Runs the hot rank update query in batches until all rows have been processed.
/// In `where_clause` and `set_clause`, "a" will refer to the current aggregates table.
/// Locked rows are skipped in order to prevent deadlocks (they will likely get updated on the next
/// run)
async fn process_ranks_in_batches(
  conn: &mut AsyncPgConnection,
  table_name: &str,
  where_clause: &str,
  set_clause: &str,
) -> FastJobResult<()> {
  let process_start_time: DateTime<Utc> = Utc.timestamp_opt(0, 0).single().unwrap_or_default();

  let update_batch_size = 1000; // Bigger batches than this tend to cause seq scans
  let mut processed_rows_count = 0;
  let mut previous_batch_result = Some(process_start_time);
  while let Some(previous_batch_last_published) = previous_batch_result {
    // Raw `sql_query` is used as a performance optimization - Diesel does not support doing this
    // in a single query (neither as a CTE, nor using a subquery)
    let updated_rows = sql_query(format!(
      r#"WITH batch AS (SELECT a.id
               FROM {table_name} a
               WHERE a.published_at > $1 AND ({where_clause})
               ORDER BY a.published_at
               LIMIT $2
               FOR UPDATE SKIP LOCKED)
         UPDATE {table_name} a {set_clause}
             FROM batch WHERE a.id = batch.id RETURNING a.published_at;
    "#,
    ))
    .bind::<Timestamptz, _>(previous_batch_last_published)
    .bind::<Integer, _>(update_batch_size)
    .get_results::<HotRanksUpdateResult>(conn)
    .await
    .map_err(|e| {
      FastJobErrorType::Unknown(format!("Failed to update {} hot_ranks: {}", table_name, e))
    })?;

    processed_rows_count += updated_rows.len();
    previous_batch_result = updated_rows.last().map(|row| row.published_at);
  }
  info!(
    "Finished process_hot_ranks_in_batches execution for {} (processed {} rows)",
    table_name, processed_rows_count
  );
  Ok(())
}

/// Post aggregates is a special case, since it needs to join to the community_aggregates
/// table, to get the active monthly user counts.
async fn process_post_aggregates_ranks_in_batches(conn: &mut AsyncPgConnection) -> FastJobResult<()> {
  let process_start_time: DateTime<Utc> = Utc.timestamp_opt(0, 0).single().unwrap_or_default();

  let update_batch_size = 1000; // Bigger batches than this tend to cause seq scans
  let mut processed_rows_count = 0;
  let mut previous_batch_result = Some(process_start_time);
  while let Some(previous_batch_last_published) = previous_batch_result {
    let updated_rows = sql_query(
      r#"WITH batch AS (SELECT pa.id
           FROM post pa
           WHERE pa.published_at > $1
           AND (pa.hot_rank != 0 OR pa.hot_rank_active != 0)
           ORDER BY pa.published_at
           LIMIT $2
           FOR UPDATE SKIP LOCKED)
      UPDATE post pa
      SET hot_rank = r.hot_rank(pa.score, pa.published_at),
          hot_rank_active = r.hot_rank(pa.score, pa.newest_comment_time_necro_at),
          scaled_rank = r.scaled_rank(pa.score, pa.published_at, ca.interactions_month)
      FROM batch, community ca
      WHERE pa.id = batch.id
      AND pa.community_id = ca.id
      RETURNING pa.published_at;
"#,
    )
    .bind::<Timestamptz, _>(previous_batch_last_published)
    .bind::<Integer, _>(update_batch_size)
    .get_results::<HotRanksUpdateResult>(conn)
    .await
    .map_err(|e| {
      FastJobErrorType::Unknown(format!("Failed to update post_aggregates hot_ranks: {}", e))
    })?;

    processed_rows_count += updated_rows.len();
    previous_batch_result = updated_rows.last().map(|row| row.published_at);
  }
  info!(
    "Finished process_hot_ranks_in_batches execution for {} (processed {} rows)",
    "post_aggregates", processed_rows_count
  );
  Ok(())
}

async fn delete_expired_captcha_answers(pool: &mut DbPool<'_>) -> FastJobResult<()> {
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
async fn active_counts(pool: &mut DbPool<'_>) -> FastJobResult<()> {
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

/// Set banned to false after ban expires
async fn update_banned_when_expired(pool: &mut DbPool<'_>) -> FastJobResult<()> {
  info!("Updating banned column if it expires ...");
  let mut conn = get_conn(pool).await?;

  uplete::new(
    community_actions::table.filter(community_actions::ban_expires_at.lt(now().nullable())),
  )
  .set_null(community_actions::received_ban_at)
  .set_null(community_actions::ban_expires_at)
  .as_query()
  .execute(&mut conn)
  .await?;
  
  Ok(())
}

/// Set banned to false after ban expires
/// Find all unpublished posts with scheduled date in the future, and publish them.
async fn publish_scheduled_posts(context: &Data<FastJobContext>) -> FastJobResult<()> {
  let pool = &mut context.pool();
  let mut conn = get_conn(pool).await?;

  let not_community_banned_action = community_actions::table
    .find((person::id, community::id))
    .filter(community_actions::received_ban_at.is_not_null());
  

  let scheduled_posts: Vec<_> = post::table
    .inner_join(community::table)
    .inner_join(person::table)
    // find all posts which have scheduled_publish_time that is in the  past
    .filter(post::scheduled_publish_time_at.is_not_null())
    .filter(coalesce(post::scheduled_publish_time_at, now()).lt(now()))
    // make sure the post, person and community are still around
    .filter(not(post::deleted.or(post::removed)))
    .filter(not(person::deleted))
    .filter(not(community::removed.or(community::deleted)))
    // ensure that user isnt banned from community
    .filter(not(exists(not_community_banned_action)))
    // ensure that user isnt banned from local
    .select((post::all_columns, community::all_columns))
    .get_results::<(Post, Community)>(&mut conn)
    .await?;

  for (post, _community) in scheduled_posts {
    // mark post as published in db
    let form = PostUpdateForm {
      scheduled_publish_time_at: Some(None),
      ..Default::default()
    };
    Post::update(&mut context.pool(), post.id, &form).await?;

    // send out post via federation and webmention
    let send_activity = SendActivityData::CreatePost(post.clone());
    ActivityChannel::submit_activity(send_activity, context)?;
  }
  Ok(())
}