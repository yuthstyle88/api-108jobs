use crate::newtypes::ChatRoomId;
use crate::{
  newtypes::ChatMessageId,
  source::chat_message::{ChatMessage, ChatMessageInsertForm, ChatMessageUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use app_108jobs_db_schema_file::schema::chat_message;
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use diesel::dsl::update;
use diesel::{dsl::insert_into, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;

impl Crud for ChatMessage {
  type InsertForm = ChatMessageInsertForm;
  type UpdateForm = ChatMessageUpdateForm;
  type IdType = ChatMessageId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(chat_message::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateChatMessage)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    update(chat_message::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateChatMessage)
  }
}

impl ChatMessage {
  /// Fetch the most recent message for a room, or None if no messages.
  pub async fn last_by_room(
    pool: &mut DbPool<'_>,
    room: ChatRoomId,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    let opt = chat_message::table
      .filter(chat_message::room_id.eq(room))
      .order(chat_message::created_at.desc())
      .first::<Self>(conn)
      .await
      .optional()
      .with_fastjob_type(FastJobErrorType::CouldntUpdateChatMessage)?;
    Ok(opt)
  }
  pub async fn bulk_insert(
    pool: &mut DbPool<'_>,
    forms: &[ChatMessageInsertForm],
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    insert_into(chat_message::table)
      .values(forms)
      .get_results(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateChatMessage)
  }

  /// Fetch the most recent messages for a room (default order: oldest -> newest)
  pub async fn list_by_room(
    pool: &mut DbPool<'_>,
    room: ChatRoomId,
    limit_num: i64,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    let mut rows = chat_message::table
      .filter(chat_message::room_id.eq(room))
      .order(chat_message::created_at.desc())
      .limit(limit_num)
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateChatMessage)?;

    // Reverse to chronological (oldest first)
    rows.reverse();
    Ok(rows)
  }

  /// Fetch paginated messages for a room (page starts at 1). Returns oldest->newest within the page
  pub async fn list_by_room_paged(
    pool: &mut DbPool<'_>,
    room: ChatRoomId,
    page: i64,
    page_size: i64,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    let safe_page = if page < 1 { 1 } else { page };
    let safe_size = if page_size <= 0 {
      20
    } else {
      page_size.min(100)
    };
    let offset = (safe_page - 1) * safe_size;

    let mut rows = chat_message::table
      .filter(chat_message::room_id.eq(room))
      .order(chat_message::created_at.desc())
      .limit(safe_size)
      .offset(offset)
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateChatMessage)?;
    rows.reverse();
    Ok(rows)
  }

  /// Cursor-based pagination: fetch messages after a given message id (strictly greater),
  /// ordered oldest -> newest. If after_id is None, starts from the very first message.
  pub async fn list_by_room_after_id(
    pool: &mut DbPool<'_>,
    room: ChatRoomId,
    after_id: Option<ChatMessageId>,
    limit_num: i64,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    let mut query = chat_message::table
      .filter(chat_message::room_id.eq(room))
      .into_boxed();
    if let Some(aid) = after_id {
      query = query.filter(chat_message::id.gt(aid));
    }

    // Order ascending by id to return oldest -> newest for the page
    let rows = RunQueryDsl::load(query.order(chat_message::id.asc()).limit(limit_num), conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateChatMessage)?;
    Ok(rows)
  }
}

// ============================================================================
// DB-backed tests for chat message persistence used by the WS history,
// flush-from-Redis, and pagination paths.
//
// Coverage:
//   * bulk_insert preserves order and returns the inserted rows.
//   * list_by_room returns messages oldest -> newest (reversed internally).
//   * list_by_room_after_id powers cursor pagination — returns only ids
//     strictly greater than the cursor.
//   * last_by_room finds the latest row by created_at.
//   * Cross-room isolation: messages in room A are never returned for room B.
//
// NOTE on `pending_sender_ack`: the `Crud::update` impl is `unimplemented!()`
// per the audit, so this layer cannot test the pending-ack flow. The flush
// path that touches it (`flush_room_and_update_last_message`) is exercised
// indirectly by the WS broker — see report.
// ============================================================================
#[cfg(test)]
mod tests {
  use super::*;
  use crate::newtypes::LocalUserId;
  use crate::source::chat_room::{ChatRoom, ChatRoomInsertForm};
  use crate::source::instance::Instance;
  use crate::source::person::{Person, PersonInsertForm};
  use crate::test_data::pool_for_tests;
  use app_108jobs_db_schema_file::schema::local_user;
  use chrono::{Duration, Utc};
  use diesel::ExpressionMethods;
  use diesel_async::RunQueryDsl;
  use serial_test::serial;

  struct ChatCtx {
    instance_id: crate::newtypes::InstanceId,
    sender_id: LocalUserId,
    room_id: ChatRoomId,
  }

  async fn fixture(pool: &mut DbPool<'_>) -> ChatCtx {
    let inst = Instance::read_or_create(pool, format!("cm-test-{}.tld", uuid::Uuid::new_v4()))
      .await
      .expect("create instance");
    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let suffix_short = &suffix[..8];
    let (p_form, _w) =
      PersonInsertForm::test_form_with_wallet(pool, inst.id, &format!("cm-{suffix_short}"))
        .await
        .expect("test_form_with_wallet");
    let person = Person::create(pool, &p_form).await.expect("create person");
    let local_user_id: i32 = {
      let conn = &mut get_conn(pool).await.expect("get conn");
      diesel::insert_into(local_user::table)
        .values((
          local_user::person_id.eq(person.id),
          local_user::password_encrypted.eq::<Option<String>>(None),
        ))
        .returning(local_user::id)
        .get_result(conn)
        .await
        .expect("insert local_user")
    };

    let room_id = ChatRoomId(format!("cm-room-{}-{}", std::process::id(), suffix_short));
    let _ = ChatRoom::create(
      pool,
      &ChatRoomInsertForm {
        id: room_id.clone(),
        room_name: format!("chat-msg test {suffix_short}"),
        created_at: Utc::now(),
        updated_at: None,
        post_id: None,
        current_comment_id: None,
      },
    )
    .await
    .expect("create room");

    ChatCtx {
      instance_id: inst.id,
      sender_id: LocalUserId(local_user_id),
      room_id,
    }
  }

  /// `msg_ref_id` carries a UNIQUE constraint; tag each generated ref with
  /// the caller-supplied tag plus a UUID so re-runs (and parallel tests) do
  /// not collide on the index.
  fn message_form(
    ctx: &ChatCtx,
    ref_tag: &str,
    content: &str,
    at_offset_secs: i64,
  ) -> ChatMessageInsertForm {
    ChatMessageInsertForm {
      msg_ref_id: Some(format!("{ref_tag}-{}", uuid::Uuid::new_v4())),
      room_id: ctx.room_id.clone(),
      sender_id: Some(ctx.sender_id),
      content: Some(content.to_string()),
      status: 1,
      created_at: Some(Utc::now() + Duration::seconds(at_offset_secs)),
      updated_at: None,
    }
  }

  async fn cleanup(pool: &mut DbPool<'_>, instance_id: crate::newtypes::InstanceId) {
    let _ = Instance::delete(pool, instance_id).await;
  }

  /// bulk_insert persists every row and returns them.
  /// list_by_room returns them oldest -> newest.
  #[tokio::test]
  #[serial]
  async fn bulk_insert_and_list_returns_oldest_first() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = fixture(pool).await;

    let forms = vec![
      message_form(&ctx, "r1", "first", 0),
      message_form(&ctx, "r2", "second", 1),
      message_form(&ctx, "r3", "third", 2),
    ];
    let inserted = ChatMessage::bulk_insert(pool, &forms).await.expect("bulk");
    assert_eq!(inserted.len(), 3);

    let listed = ChatMessage::list_by_room(pool, ctx.room_id.clone(), 10)
      .await
      .expect("list");
    let contents: Vec<&str> = listed.iter().map(|m| m.content.as_str()).collect();
    assert_eq!(
      contents,
      vec!["first", "second", "third"],
      "list_by_room must return oldest first"
    );

    let last = ChatMessage::last_by_room(pool, ctx.room_id.clone())
      .await
      .expect("last");
    assert_eq!(last.map(|m| m.content), Some("third".to_string()));
    cleanup(pool, ctx.instance_id).await;
  }

  /// list_by_room_after_id powers cursor pagination — returns only rows with
  /// id strictly greater than the supplied cursor.
  #[tokio::test]
  #[serial]
  async fn list_after_id_excludes_cursor_row() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = fixture(pool).await;
    let inserted = ChatMessage::bulk_insert(
      pool,
      &[
        message_form(&ctx, "a", "1", 0),
        message_form(&ctx, "b", "2", 1),
        message_form(&ctx, "c", "3", 2),
      ],
    )
    .await
    .expect("bulk");
    let mid = inserted[1].id;

    let after_mid = ChatMessage::list_by_room_after_id(pool, ctx.room_id.clone(), Some(mid), 10)
      .await
      .expect("after");
    assert_eq!(after_mid.len(), 1, "only the row strictly after mid");
    assert_eq!(after_mid[0].content, "3");

    let from_start = ChatMessage::list_by_room_after_id(pool, ctx.room_id.clone(), None, 10)
      .await
      .expect("from start");
    assert_eq!(from_start.len(), 3);
    cleanup(pool, ctx.instance_id).await;
  }

  /// Messages in one room must not leak into another room's listing.
  #[tokio::test]
  #[serial]
  async fn list_by_room_isolates_rooms() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = fixture(pool).await;

    // Second room in the same instance.
    let other_room_id = ChatRoomId(format!("cm-other-{}", uuid::Uuid::new_v4().simple()));
    let _ = ChatRoom::create(
      pool,
      &ChatRoomInsertForm {
        id: other_room_id.clone(),
        room_name: "other".to_string(),
        created_at: Utc::now(),
        updated_at: None,
        post_id: None,
        current_comment_id: None,
      },
    )
    .await
    .expect("create other");

    let _ = ChatMessage::bulk_insert(pool, &[message_form(&ctx, "x", "in-target", 0)])
      .await
      .expect("target");
    let other_form = ChatMessageInsertForm {
      msg_ref_id: Some(format!("y-{}", uuid::Uuid::new_v4())),
      room_id: other_room_id.clone(),
      sender_id: Some(ctx.sender_id),
      content: Some("in-other".to_string()),
      status: 1,
      created_at: Some(Utc::now()),
      updated_at: None,
    };
    let _ = ChatMessage::bulk_insert(pool, &[other_form])
      .await
      .expect("other");

    let target_rows = ChatMessage::list_by_room(pool, ctx.room_id.clone(), 100)
      .await
      .expect("list target");
    assert!(target_rows.iter().all(|m| m.content == "in-target"));
    cleanup(pool, ctx.instance_id).await;
  }
}
