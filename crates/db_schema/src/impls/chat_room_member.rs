use crate::{
  newtypes::{ChatRoomId, LocalUserId},
  source::chat_room_member::ChatRoomMember,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::count_distinct, insert_into, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::chat_room_member;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use crate::source::chat_room_member::{ChatRoomMemberInsertForm, ChatRoomMemberUpdateForm};
use crate::traits::Crud;

impl Crud for ChatRoomMember {
  type InsertForm = ChatRoomMemberInsertForm;
  type UpdateForm = ChatRoomMemberUpdateForm;
  type IdType = (ChatRoomId, LocalUserId);

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(chat_room_member::table)
        .values(form)
        .get_result::<Self>(conn)
        .await
        .with_fastjob_type(FastJobErrorType::CouldntCreateChatRoomMember)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(chat_room_member::table.find((id.0, id.1)))
        .set(form)
        .get_result::<Self>(conn)
        .await
        .with_fastjob_type(FastJobErrorType::CouldntUpdateChatRoomMember)
  }
}

impl ChatRoomMember {
  pub async fn find_room_by_members(
    pool: &mut DbPool<'_>,
    member1: LocalUserId,
    member2: LocalUserId,
  ) -> FastJobResult<Option<ChatRoomId>> {
    let conn = &mut get_conn(pool).await?;

    let results: Vec<i32> = chat_room_member::table
      .filter(chat_room_member::user_id.eq_any(vec![member1.0, member2.0]))
      .select(chat_room_member::room_id)
      .group_by(chat_room_member::room_id)
      .having(count_distinct(chat_room_member::user_id).eq(2))
      .load(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;

    Ok(results.into_iter().next().map(ChatRoomId))
  }
}
