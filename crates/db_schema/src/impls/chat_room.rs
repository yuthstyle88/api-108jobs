use crate::{
    source::chat_room::{ChatRoom, ChatRoomInsertForm, ChatRoomUpdateForm},
    traits::Crud,
    utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, QueryDsl};
use diesel::dsl::{exists, select};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::chat_room;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use crate::newtypes::ChatRoomId;

impl Crud for ChatRoom {
    type InsertForm = ChatRoomInsertForm;
    type UpdateForm = ChatRoomUpdateForm;
    type IdType = ChatRoomId;

    async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
        let conn = &mut get_conn(pool).await?;
        insert_into(chat_room::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntCreateChatRoom)
    }

    async fn update(
        pool: &mut DbPool<'_>,
        id: Self::IdType,
        form: &Self::UpdateForm,
    ) -> FastJobResult<Self> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(chat_room::table.find(id))
            .set(form)
            .get_result::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntUpdateChatRoom)
    }
}

impl ChatRoom {
    pub async fn bulk_insert(
        pool: &mut DbPool<'_>,
        forms: &[ChatRoomInsertForm],
    ) -> FastJobResult<Vec<Self>> {
        let conn = &mut get_conn(pool).await?;

        insert_into(chat_room::table)
            .values(forms)
            .get_results(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntCreateChatRoom)
    }

    pub async fn exists(pool: &mut DbPool<'_>, id: ChatRoomId) -> FastJobResult<bool> {
        let conn = &mut get_conn(pool).await?;

        let exists_query = select(exists(chat_room::table.find(id)));

        let found = exists_query
            .get_result::<bool>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::DatabaseError)?;

        Ok(found)
    }
}
