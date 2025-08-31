use crate::newtypes::{ChatRoomId, LocalUserId};
use crate::{
  source::chat_participant::{
    ChatParticipant, ChatParticipantInsertForm, ChatParticipantUpdateForm,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::dsl::update;
use diesel::{dsl::insert_into, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::chat_participant;
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
  pub async fn bulk_insert(
    pool: &mut DbPool<'_>,
    forms: &[ChatParticipantInsertForm],
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    insert_into(chat_participant::table)
      .values(forms)
      .get_results(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateChatRoom)
  }
}
