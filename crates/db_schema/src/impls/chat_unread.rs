use crate::newtypes::{ChatRoomId, LocalUserId};
use crate::source::chat_unread::{ChatUnread, ChatUnreadUpsertForm};
use crate::utils::{get_conn, DbPool};
use chrono::{DateTime, Utc};
use diesel::dsl::{insert_into, now, update};
use diesel::prelude::*;
use diesel::upsert::excluded;
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::chat_unread;
use lemmy_utils::error::{FastJobErrorType, FastJobResult, FastJobErrorExt};
use diesel::sql_types::{Integer, Nullable, Timestamptz, Text, Varchar};
use diesel::sql_query;

impl ChatUnread {
  /// Increment unread_count by 1 for (user, room), and update last_message_*.
  /// Creates the row if missing.
  pub async fn increment_unread(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    room_id: ChatRoomId,
    last_message_id: Option<String>,
    last_message_at: Option<DateTime<Utc>>,
  ) -> FastJobResult<ChatUnread> {
    let conn = &mut get_conn(pool).await?;

    let form = ChatUnreadUpsertForm {
      local_user_id: user_id,
      room_id: room_id.clone(),
      unread_count: 1,
      last_message_id,
      last_message_at,
    };

    let q = insert_into(chat_unread::table)
        .values(&form)
        .on_conflict((chat_unread::local_user_id, chat_unread::room_id))
        .do_update()
        .set((
          chat_unread::unread_count.eq(chat_unread::unread_count + 1),
          chat_unread::last_message_id.eq(excluded(chat_unread::last_message_id)),
          chat_unread::last_message_at.eq(excluded(chat_unread::last_message_at)),
          chat_unread::updated_at.eq(now),
        ))
        .returning(ChatUnread::as_returning());

    let row: ChatUnread = q
        .get_result(conn)
        .await
        .with_fastjob_type(FastJobErrorType::CouldntUpdateChatUnread)?;

    Ok(row)
  }

  /// Reset unread_count to 0 for (user, room). Keeps last_message_* intact.
  pub async fn reset_unread(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    room_id: ChatRoomId,
  ) -> FastJobResult<ChatUnread> {
    let conn = &mut get_conn(pool).await?;

    // Ensure the row exists; if not, insert with zero.
    let _ = insert_into(chat_unread::table)
        .values((
          chat_unread::local_user_id.eq(user_id),
          chat_unread::room_id.eq(room_id.clone()),
          chat_unread::unread_count.eq(0_i32),
        ))
        .on_conflict_do_nothing()
        .execute(conn)
        .await
        .with_fastjob_type(FastJobErrorType::CouldntUpdateChatUnread)?;

    let q = update(chat_unread::table.find((user_id, room_id)))
        .set((
          chat_unread::unread_count.eq(0_i32),
          chat_unread::updated_at.eq(now),
        ))
        .returning(ChatUnread::as_returning());

    let row: ChatUnread = q
        .get_result(conn)
        .await
        .with_fastjob_type(FastJobErrorType::CouldntUpdateChatUnread)?;

    Ok(row)
  }

  /// Get current unread row if exists.
  pub async fn get_unread_for_user_room(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    room_id: ChatRoomId,
  ) -> FastJobResult<Option<ChatUnread>> {
    let conn = &mut get_conn(pool).await?;
    let q = chat_unread::table.find((user_id, room_id));
    let row = q
        .get_result::<ChatUnread>(conn)
        .await
        .optional()
        .with_fastjob_type(FastJobErrorType::CouldntUpdateChatUnread)?;
    Ok(row)
  }

  /// Fetch unread snapshot for a user across all rooms they participate in.
  /// Returns tuples of (room_id, unread_count, last_message_id, last_message_at) for every room.
  /// 
  pub async fn unread_snapshot_for_user(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
  ) -> FastJobResult<Vec<(ChatRoomId, i32, Option<String>, Option<DateTime<Utc>>)>> {
    let conn = &mut get_conn(pool).await?;

    #[derive(QueryableByName)]
    struct SnapshotRow {
      #[diesel(sql_type = Varchar)]
      room_id: String,
      #[diesel(sql_type = Integer)]
      unread_count: i32,
      #[diesel(sql_type = Nullable<Varchar>)]
      last_message_id: Option<String>,
      #[diesel(sql_type = Nullable<Timestamptz>)]
      last_message_at: Option<DateTime<Utc>>,
    }

    let q = sql_query(
      r#"
      SELECT p.room_id,
             COALESCE(u.unread_count, 0)               AS unread_count,
             u.last_message_id,
             u.last_message_at
      FROM chat_participant p
      LEFT JOIN chat_unread u
        ON u.local_user_id = $1
       AND u.room_id = p.room_id
      WHERE p.member_id = $1
      ORDER BY COALESCE(u.last_message_at, to_timestamp(0)) DESC
      "#,
    )
    .bind::<Integer, _>(user_id.0);

    let rows: Vec<SnapshotRow> = q
      .get_results(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateChatUnread)?;

    Ok(rows
      .into_iter()
      .map(|r| (ChatRoomId(r.room_id), r.unread_count, r.last_message_id, r.last_message_at))
      .collect())
  }

  /// Bulk increment unread_count by 1 for all members of a room (excluding sender if provided).
  /// Returns pairs of (local_user_id, unread_count) for each affected recipient after the update.
  pub async fn bulk_increment_for_room(
    pool: &mut DbPool<'_>,
    room_id: ChatRoomId,
    sender_id: Option<LocalUserId>,
    last_message_id: Option<String>,
    last_message_at: Option<DateTime<Utc>>,
  ) -> FastJobResult<Vec<(LocalUserId, i32)>> {
    let conn = &mut get_conn(pool).await?;

    #[derive(QueryableByName)]
    struct BulkResult {
      #[diesel(sql_type = Integer)]
      local_user_id: i32,
      #[diesel(sql_type = Integer)]
      unread_count: i32,
    }

    // Using SQL to leverage INSERT ... SELECT ... ON CONFLICT ... RETURNING
    // Note: chat_participant uses columns (room_id, member_id)
    let q = sql_query(
      r#"
      INSERT INTO chat_unread (local_user_id, room_id, unread_count, last_message_id, last_message_at)
      SELECT p.member_id, $1 AS room_id, 1, $2, $3
      FROM chat_participant p
      WHERE p.room_id = $1 AND ($4 IS NULL OR p.member_id <> $4)
      ON CONFLICT (local_user_id, room_id) DO UPDATE
      SET unread_count    = chat_unread.unread_count + 1,
          last_message_id = EXCLUDED.last_message_id,
          last_message_at = EXCLUDED.last_message_at,
          updated_at      = now()
      RETURNING local_user_id, unread_count
      "#,
    )
    .bind::<Text, _>(room_id.0)
    .bind::<Nullable<Text>, _>(last_message_id)
    .bind::<Nullable<Timestamptz>, _>(last_message_at)
    .bind::<Nullable<Integer>, _>(sender_id.map(|s| s.0));

    let rows: Vec<BulkResult> = q
      .get_results(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateChatUnread)?;

    let mapped = rows
      .into_iter()
      .map(|r| (LocalUserId(r.local_user_id), r.unread_count))
      .collect::<Vec<_>>();

    Ok(mapped)
  }
}
