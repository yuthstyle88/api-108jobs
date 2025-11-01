use crate::api::{GetChatRoomRequest, ListUserChatRooms};
use crate::{ChatMessageView, ChatParticipantView, ChatRoomView};
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::{stream, StreamExt, TryStreamExt};
use lemmy_db_schema::source::person::Person;
use lemmy_db_schema::source::workflow::Workflow;
use lemmy_db_schema::{
  newtypes::{ChatMessageId, ChatRoomId, LocalUserId, PaginationCursor},
  source::{
    chat_message::ChatMessage, chat_participant::ChatParticipant, chat_room::ChatRoom,
    comment::Comment, post::Post,
  },
  traits::{Crud, PaginationCursorBuilder},
  try_join_with_pool,
  utils::{get_conn, limit_fetch, DbPool},
};
use lemmy_db_schema_file::schema::{chat_message, chat_participant, chat_room, local_user, person};
use lemmy_utils::error::{FastJobError, FastJobErrorType, FastJobResult};

/// Cursor support for chat message pagination
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

/// ChatRoomView – parallel loading of a single room
impl ChatRoomView {
  pub async fn read(pool: &mut DbPool<'_>, id: ChatRoomId) -> FastJobResult<Self> {
    let room = ChatRoom::read(pool, id).await?;
    let ChatRoom {
      id: room_id,
      post_id,
      current_comment_id,
      ..
    } = room.clone();

    let rid_for_msg = room_id.clone();
    let rid_for_part = room_id.clone();
    let rid_for_workflow = room_id.clone();

    let (post, current_comment, last_message, participants, workflow) = try_join_with_pool!(
        pool => (
            |pool| async move {
                match post_id {
                    Some(pid) => Post::read(pool, pid).await.map(Some),
                    None => Ok(None),
                }
            },
            |pool| async move {
                match current_comment_id {
                    Some(cid) => Comment::read(pool, cid).await.map(Some),
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
      current_comment,
      last_message,
      workflow,
    })
  }
  /// List all rooms a user participates in – **in parallel**.
  pub async fn list_for_user(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    limit: i64,
  ) -> FastJobResult<Vec<Self>> {
    // Get all rooms this user participates in
    let participants = ChatParticipant::list_rooms_for_user(pool, user_id, limit).await?;
    if participants.is_empty() {
      return Ok(vec![]);
    }

    /// Bounded parallelism when using a real pool; sequential when inside a transaction/Conn
    /// If you have 3 rooms → all 3 run at once.
    /// If you have 20 rooms → it runs 8 first, then starts new ones as old ones finish.
    const MAX_CONCURRENCY: usize = 8;
    match pool {
      DbPool::Pool(__pool) => {
        let indexed = participants.into_iter().enumerate();
        let stream = stream::iter(indexed.map(|(idx, p)| {
          let __pool = *__pool;
          async move {
            let mut dbp = DbPool::Pool(__pool);
            let room = ChatRoomView::read(&mut dbp, p.id).await?;
            Ok::<(usize, Self), FastJobError>((idx, room))
          }
        }));

        let mut collected: Vec<(usize, Self)> = stream
          .buffer_unordered(MAX_CONCURRENCY)
          .try_collect()
          .await?;
        collected.sort_by(|(_, a), (_, b)| {
          let a_time = a
            .last_message
            .as_ref()
            .and_then(|m| Some(m.created_at))
            .unwrap_or_default();
          let b_time = b
            .last_message
            .as_ref()
            .and_then(|m| Some(m.created_at))
            .unwrap_or_default();
          b_time.cmp(&a_time)
        });
        Ok(collected.into_iter().map(|(_, room)| room).collect())
      }
      DbPool::Conn(__conn) => {
        let mut out = Vec::with_capacity(participants.len());
        for p in participants {
          let mut dbp = DbPool::Conn(__conn);
          let room = ChatRoomView::read(&mut dbp, p.id).await?;
          out.push(room);
        }
        out.sort_by(|a, b| {
          let a_time = a
            .last_message
            .as_ref()
            .and_then(|m| Some(m.created_at))
            .unwrap_or_default();
          let b_time = b
            .last_message
            .as_ref()
            .and_then(|m| Some(m.created_at))
            .unwrap_or_default();
          b_time.cmp(&a_time)
        });
        Ok(out)
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
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
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
