use crate::api::{GetChatRoomRequest, ListUserChatRooms};
use crate::{ChatMessageView, ChatParticipantView, ChatRoomView};
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::source::person::Person;
use lemmy_db_schema::{
  newtypes::{ChatMessageId, ChatRoomId, PaginationCursor},
  source::{
    chat_message::ChatMessage, chat_participant::ChatParticipant, chat_room::ChatRoom,
    comment::Comment, post::Post,
  },
  traits::{Crud, PaginationCursorBuilder},
  utils::{get_conn, limit_fetch, DbPool},
};
use lemmy_db_schema_file::schema::{chat_message, chat_participant, chat_room, local_user, person};
use lemmy_utils::error::{FastJobError, FastJobErrorType, FastJobResult};

impl PaginationCursorBuilder for ChatMessageView {
  type CursorData = ChatMessage;
  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new_single('M', self.message.id.0.try_into().unwrap())
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    let id = cursor.first_id()?;
    ChatMessage::read(pool, ChatMessageId(id.into())).await
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

    // read optional post without holding a borrowed connection
    let post = match room.post_id.clone() {
      Some(pid) => Some(Post::read(pool, pid).await?),
      None => None,
    };
    // read optional current comment
    let current_comment = match room.current_comment_id.clone() {
      Some(cid) => Some(Comment::read(pool, cid).await?),
      None => None,
    };

    // read participants
    let rid = room.id.clone();
    let parts = ChatParticipantView::list_for_room(pool, rid).await?;

    Ok(ChatRoomView {
      room,
      participants: parts,
      post,
      current_comment,
    })
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

impl ChatParticipantView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    use lemmy_db_schema_file::schema::{chat_participant, local_user, person};

    // Join chat_participant -> local_user -> person
    let local_user_join = local_user::table.on(chat_participant::member_id.eq(local_user::id));
    let person_join = person::table.on(local_user::person_id.eq(person::id));

    chat_participant::table
      .inner_join(local_user_join)
      .inner_join(person_join)
  }

  pub async fn list_for_room(
    pool: &mut DbPool<'_>,
    room_id: ChatRoomId,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    use diesel::SelectableHelper;
    let results = Self::joins()
      .filter(chat_participant::room_id.eq(room_id))
      .select((ChatParticipant::as_select(), Person::as_select()))
      .load::<(ChatParticipant, Person)>(conn)
      .await?;

    Ok(
      results
        .into_iter()
        .map(|(participant, member_person)| ChatParticipantView {
          participant,
          member_person,
        })
        .collect(),
    )
  }
}
