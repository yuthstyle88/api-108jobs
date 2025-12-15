use crate::api::{GetChatRoomRequest, ListUserChatRooms};
use crate::{ChatMessageView, ChatParticipantView, ChatRoomView};
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::{StreamExt, TryStreamExt};
use lemmy_db_schema::source::workflow::Workflow;
use lemmy_db_schema::{
  newtypes::{ChatMessageId, ChatRoomId, LocalUserId, PaginationCursor},
  source::{chat_message::ChatMessage, chat_room::ChatRoom},
  traits::{Crud, PaginationCursorBuilder},
  try_join_with_pool,
  utils::{get_conn, limit_fetch, DbPool},
};
use lemmy_db_schema_file::schema::{chat_message, chat_participant, chat_room, local_user, person};
use lemmy_db_views_post::PostPreview;
use lemmy_utils::error::{FastJobError, FastJobErrorType, FastJobResult};

/// Cursor support for chat message pagination
impl PaginationCursorBuilder for ChatMessageView {
  type CursorData = ChatMessage;

  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new_i64('M', self.message.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    let id = cursor.as_i64()?;
    ChatMessage::read(pool, ChatMessageId(id)).await
  }
}

/// ChatMessageView
impl ChatMessageView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    let local_user_join = local_user::table.on(chat_message::sender_id.eq(local_user::id));
    let room_join = chat_room::table.on(chat_message::room_id.eq(chat_room::id));
    chat_message::table
      .inner_join(local_user_join)
      .inner_join(room_join)
  }

  // Production-safe: simple read with error handling
  pub async fn read(pool: &mut DbPool<'_>, id: ChatMessageId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(chat_message::id.eq(id))
      .select(Self::as_select())
      .first(conn)
      .await
      .map_err(|_| FastJobErrorType::NotFound.into())
  }

  /// Pagination-safe message listing
  pub async fn list_for_room(
    pool: &mut DbPool<'_>,
    room_id: ChatRoomId,
    limit: Option<i64>,
    cursor_data: Option<ChatMessage>,
    page_back: Option<bool>,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(limit)?;

    let mut query = Self::joins()
      .filter(chat_message::room_id.eq(room_id))
      .select(Self::as_select())
      .into_boxed();

    if let Some(cursor) = cursor_data {
      if page_back.unwrap_or(false) {
        query = query.filter(chat_message::id.lt(cursor.id));
      } else {
        query = query.filter(chat_message::id.gt(cursor.id));
      }
    }

    let res = query
      .order_by(chat_message::id.desc())
      .limit(limit)
      .load::<Self>(conn)
      .await?;
    Ok(res)
  }
}

impl PaginationCursorBuilder for ChatRoomView {
  type CursorData = i64;

  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new_i64('R', self.room.serial_id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    _pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    cursor.as_i64()
  }
}

/// ChatRoomView – parallel loading of a single room
impl ChatRoomView {
  pub async fn read(pool: &mut DbPool<'_>, id: ChatRoomId) -> FastJobResult<Self> {
    let room = ChatRoom::read(pool, id).await?;
    let ChatRoom {
      id: room_id,
      post_id,
      ..
    } = room.clone();

    let rid_for_msg = room_id.clone();
    let rid_for_part = room_id.clone();
    let rid_for_workflow = room_id.clone();

    let (post, last_message, participants, workflow) = try_join_with_pool!(
        pool => (
            |pool| async move {
                match post_id {
                  Some(pid) => PostPreview::read(pool, pid).await.map(Some),
                  None => Ok(None),
                }
            },
            |pool| async move {
                ChatMessage::last_by_room(pool, rid_for_msg).await
            },
            |pool| async move {
                ChatParticipantView::list_for_room(pool, rid_for_part).await
            },
            |pool| async move {
                Workflow::get_current_by_room_id(pool, rid_for_workflow).await
            }
        )
    )?;

    Ok(Self {
      room,
      participants,
      post,
      last_message,
      workflow,
    })
  }
  /// List all rooms a user participates in – **in parallel**.
  pub async fn list_for_user_paginated(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    limit: Option<i64>,
    cursor: Option<PaginationCursor>,
    page_back: Option<bool>,
  ) -> FastJobResult<Vec<Self>> {
    let limit = limit_fetch(limit)?;
    let page_back = page_back.unwrap_or(false);

    // 1. Get only the serial_ids of rooms the user is in, paginated via keyset
    let serial_ids =
      Self::paginated_serial_ids(pool, user_id, limit, cursor.as_ref(), page_back).await?;

    if serial_ids.is_empty() {
      return Ok(vec![]);
    }

    // 2. Load full ChatRoomView in parallel (your existing battle-tested logic)
    let room_ids = serial_ids
      .into_iter()
      .map(|serial_id| {
        // You need a way to get ChatRoomId from serial_id.
        // Assuming you added a helper or can query by serial_id:
        // We'll do a fast lookup below
        serial_id
      })
      .collect::<Vec<_>>();

    // Fast path: lookup ChatRoomId by serial_id in one query
    let id_map: std::collections::HashMap<i64, ChatRoomId> = chat_room::table
      .filter(chat_room::serial_id.eq_any(&room_ids))
      .select((chat_room::serial_id, chat_room::id))
      .load::<(i64, ChatRoomId)>(&mut get_conn(pool).await?)
      .await?
      .into_iter()
      .collect();

    let room_ids: Vec<ChatRoomId> = room_ids
      .into_iter()
      .filter_map(|serial_id| id_map.get(&serial_id).cloned())
      .collect();

    // 3. Reuse your existing parallel loader
    Self::load_views_parallel(pool, room_ids).await
  }

  /// Core keyset pagination query — pure Diesel, indexed, blazing fast
  async fn paginated_serial_ids(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    limit: i64,
    cursor: Option<&PaginationCursor>,
    page_back: bool,
  ) -> FastJobResult<Vec<i64>> {
    let mut query = chat_participant::table
      .inner_join(chat_room::table.on(chat_participant::room_id.eq(chat_room::id)))
      .filter(chat_participant::member_id.eq(user_id))
      .select(chat_room::serial_id)
      .into_boxed();

    // Apply cursor if present
    if let Some(cur) = cursor {
      let cursor_serial = ChatRoomView::from_cursor(cur, pool).await?;
      if page_back {
        query = query.filter(chat_room::serial_id.lt(cursor_serial));
      } else {
        query = query.filter(chat_room::serial_id.gt(cursor_serial));
      }
    }

    let serial_ids: Vec<i64> = query
      .order_by(chat_room::serial_id.desc())
      .limit(limit)
      .load(&mut get_conn(pool).await?)
      .await?;

    Ok(serial_ids)
  }

  async fn load_views_parallel(
    pool: &mut DbPool<'_>,
    room_ids: Vec<ChatRoomId>,
  ) -> FastJobResult<Vec<Self>> {
    const MAX_CONCURRENCY: usize = 8;

    match pool {
      DbPool::Pool(p) => {
        let stream = futures_util::stream::iter(room_ids.into_iter().map(|room_id| {
          let p = *p;
          async move {
            let mut dbp = DbPool::Pool(p);
            ChatRoomView::read(&mut dbp, room_id).await
          }
        }));

        let views = stream
          .buffer_unordered(MAX_CONCURRENCY)
          .try_collect::<Vec<Self>>()
          .await?;

        Ok(views)
      }
      DbPool::Conn(c) => {
        let mut views = Vec::with_capacity(room_ids.len());
        for room_id in room_ids {
          let mut dbp = DbPool::Conn(*c);
          views.push(ChatRoomView::read(&mut dbp, room_id).await?);
        }
        Ok(views)
      }
    }
  }
}

/// Request validation
impl TryFrom<ListUserChatRooms> for (i64,) {
  type Error = FastJobError;
  fn try_from(value: ListUserChatRooms) -> Result<Self, Self::Error> {
    let limit = value.limit.unwrap_or(50);
    if limit <= 0 {
      return Err(FastJobErrorType::InvalidFetchLimit.into());
    }
    Ok((limit,))
  }
}

impl TryFrom<GetChatRoomRequest> for ChatRoomId {
  type Error = FastJobError;
  fn try_from(req: GetChatRoomRequest) -> Result<Self, Self::Error> {
    Ok(req.id)
  }
}

/// ChatParticipantView
impl ChatParticipantView {
  pub async fn list_for_room(
    pool: &mut DbPool<'_>,
    room_id: ChatRoomId,
  ) -> FastJobResult<Vec<Self>> {
    use diesel::prelude::{ExpressionMethods, JoinOnDsl, QueryDsl};

    let conn = &mut get_conn(pool).await?;

    let results = chat_participant::table
      .inner_join(local_user::table.on(chat_participant::member_id.eq(local_user::id)))
      .inner_join(person::table.on(local_user::person_id.eq(person::id)))
      .filter(chat_participant::room_id.eq(room_id))
      .select((
        chat_participant::member_id,
        person::id,
        person::name,
        person::display_name,
        person::avatar,
        person::available,
        chat_participant::room_id,
        chat_participant::joined_at,
      ))
      .load::<ChatParticipantView>(conn)
      .await?;

    Ok(results)
  }
}
