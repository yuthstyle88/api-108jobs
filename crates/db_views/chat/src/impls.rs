use crate::api::{GetChatRoomRequest, ListUserChatRooms};
use crate::{ChatMessageView, ChatRoomView};
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{ChatMessageId, ChatRoomId, LocalUserId, PaginationCursor},
  source::{chat_message::ChatMessage, chat_participant::ChatParticipant, chat_room::ChatRoom},
  traits::{Crud, PaginationCursorBuilder},
  utils::{get_conn, limit_fetch, DbPool},
};
use lemmy_db_schema_file::schema::{chat_message, chat_participant, chat_room, local_user};
use lemmy_utils::error::{FastJobError, FastJobErrorType, FastJobResult};

impl PaginationCursorBuilder for ChatMessageView {
  type CursorData = ChatMessage;
  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new_single('M', self.message.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    let id = cursor.first_id()?;
    ChatMessage::read(pool, ChatMessageId(id)).await
  }
}

impl ChatMessageView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    let local_user_join = local_user::table.on(chat_message::sender_id.eq(local_user::id));
    let room_join = chat_room::table.on(chat_message::room_id.eq(chat_room::id));

    chat_message::table
      .inner_join(local_user_join)
      .inner_join(room_join)
  }

  pub async fn read(pool: &mut DbPool<'_>, id: ChatMessageId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(chat_message::id.eq(id))
      .select(Self::as_select())
      .first(conn)
      .await
      .map_err(|_| FastJobErrorType::NotFound.into())
  }

  /// List messages for a room, newest first, with pagination cursor
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
        // going back in time (older messages), fetch IDs less than cursor
        query = query.filter(chat_message::id.lt(cursor.id));
      } else {
        // going forward (newer messages), fetch IDs greater than cursor
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

impl ChatRoomView {
  pub async fn read(pool: &mut DbPool<'_>, id: ChatRoomId) -> FastJobResult<Self> {
    // read room first, before borrowing a connection
    let room = ChatRoom::read(pool, id).await?;
    let conn = &mut get_conn(pool).await?;

    // read participants
    let rid = room.id.clone();
    let parts = chat_participant::table
      .filter(chat_participant::room_id.eq(rid))
      .select(ChatParticipant::as_select())
      .load::<ChatParticipant>(conn)
      .await?;

    Ok(ChatRoomView {
      room,
      participants: parts,
    })
  }

  /// List rooms for a user with participants bundled
  pub async fn list_for_user(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    limit: Option<i64>,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(limit)?;

    // first get room ids where user participates
    let rooms: Vec<ChatRoom> = chat_participant::table
      .inner_join(chat_room::table.on(chat_participant::room_id.eq(chat_room::id)))
      .filter(chat_participant::member_id.eq(user_id))
      .select(ChatRoom::as_select())
      .limit(limit)
      .load(conn)
      .await?;

    // fetch all participants for these rooms
    let room_ids: Vec<_> = rooms.iter().map(|r| r.id.clone()).collect();
    let parts: Vec<ChatParticipant> = chat_participant::table
      .filter(chat_participant::room_id.eq_any(&room_ids))
      .select(ChatParticipant::as_select())
      .load(conn)
      .await?;

    use std::collections::HashMap;
    let mut grouped: HashMap<ChatRoomId, Vec<ChatParticipant>> = HashMap::new();
    for p in parts {
      let rid = p.room_id.clone();
      grouped.entry(rid).or_default().push(p);
    }

    Ok(
      rooms
        .into_iter()
        .map(|room| ChatRoomView {
          participants: grouped.remove(&room.id).unwrap_or_default(),
          room,
        })
        .collect(),
    )
  }
}

// Validations using TryFrom for API requests
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
    // Currently only contains id; just pass through. Additional checks can be added later.
    Ok(req.id)
  }
}
