use crate::newtypes::{ChatRoomId, LocalUserId};
use crate::source::chat_participant::{ChatParticipantInsertForm, ChatParticipantUpdateForm};
use crate::traits::Crud;
use crate::{
  source::chat_participant::ChatParticipant,
  source::chat_room::ChatRoom,
  utils::{get_conn, DbPool},
};
use diesel::dsl::{insert_into, update};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{chat_participant, chat_room};
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Crud for ChatParticipant {
  type InsertForm = ChatParticipantInsertForm;
  type UpdateForm = ChatParticipantUpdateForm;
  type IdType = (ChatRoomId, LocalUserId);

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    
    insert_into(chat_participant::table)
        .values(form)
        .get_result::<Self>(conn)
        .await
        .with_fastjob_type(FastJobErrorType::CouldntCreateChatParticipant)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    
    update(chat_participant::table.find(id))
        .set(form)
        .get_result::<Self>(conn)
        .await
        .with_fastjob_type(FastJobErrorType::CouldntUpdateChatParticipant)
  }
}

impl ChatParticipant {
  /// List chat rooms for a given user (member_id), ordered by the time they joined (newest first).
  /// Returns the ChatRoom rows. Limit is clamped to [1, 100].
  pub async fn list_rooms_for_user(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    limit: i64,
  ) -> FastJobResult<Vec<ChatRoom>> {
    let conn = &mut get_conn(pool).await?;

    let mut size = if limit <= 0 { 50 } else { limit };
    if size > 100 {
      size = 100;
    }

    let rows = chat_participant::table
      .inner_join(chat_room::table.on(chat_room::id.eq(chat_participant::room_id)))
      .filter(chat_participant::member_id.eq(user_id))
      .order(chat_participant::joined_at.desc())
      .select((
        chat_room::id,
        chat_room::room_name,
        chat_room::created_at,
        chat_room::updated_at,
      ))
      .limit(size)
      .load::<ChatRoom>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntListRoomForUser)?;

    Ok(rows)
  }

  /// Ensure a participant record exists for (room_id, member_id). If not, insert it.
  pub async fn ensure_participant(
    pool: &mut DbPool<'_>,
    chat_participant_form: &ChatParticipantInsertForm
  ) -> FastJobResult<()> {
    let conn = &mut get_conn(pool).await?;

    let _ = insert_into(chat_participant::table)
      .values(chat_participant_form)
      .on_conflict((chat_participant::room_id, chat_participant::member_id))
      .do_nothing()
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntEnsureParticipant)?;

    Ok(())
  }
}
