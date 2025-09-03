use crate::{
    newtypes::ChatMessageId,
    source::chat_message::{ChatMessage, ChatMessageInsertForm, ChatMessageUpdateForm},
    traits::Crud,
    utils::{get_conn, DbPool},
};
use diesel::dsl::update;
use diesel::{dsl::insert_into, ExpressionMethods, QueryDsl, OptionalExtension};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::chat_message;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use crate::newtypes::ChatRoomId;

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
        let safe_size = if page_size <= 0 { 20 } else { page_size.min(100) };
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
        let rows = RunQueryDsl::load(
            query.order(chat_message::id.asc()).limit(limit_num),
            conn,
        )
        .await
        .with_fastjob_type(FastJobErrorType::CouldntUpdateChatMessage)?;
        Ok(rows)
    }
}
