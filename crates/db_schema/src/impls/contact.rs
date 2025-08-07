use crate::{
  newtypes::{ContactId, LocalUserId},
  source::contact::{Contact, ContactInsertForm, ContactUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, QueryDsl, ExpressionMethods};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::contact;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Crud for Contact {
  type InsertForm = ContactInsertForm;
  type UpdateForm = ContactUpdateForm;
  type IdType = ContactId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(contact::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateContact)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(contact::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateContact)
  }

  async fn delete(pool: &mut DbPool<'_>, id: Self::IdType) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(contact::table.find(id))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntDeleteContact)
  }
}

impl Contact {
  pub async fn find_by_local_user_id(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    contact::table
      .filter(contact::local_user_id.eq(local_user_id))
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntFindContact)
  }
}