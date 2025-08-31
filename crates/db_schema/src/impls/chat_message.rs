use crate::{
  newtypes::ChatMessageId,
  source::chat_message::{ChatMessage, ChatMessageInsertForm, ChatMessageUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, QueryDsl};
use diesel::dsl::update;
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::chat_message;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

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
}
