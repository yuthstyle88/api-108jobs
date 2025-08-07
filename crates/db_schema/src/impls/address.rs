use crate::{
  newtypes::{AddressId},
  source::address::{Address, AddressInsertForm, AddressUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{address};
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Crud for Address {
  type InsertForm = AddressInsertForm;
  type UpdateForm = AddressUpdateForm;
  type IdType = AddressId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
     insert_into(address::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
     .with_fastjob_type(FastJobErrorType::CouldntCreateAddress)

  }

  async fn update(pool: &mut DbPool<'_>, id: Self::IdType, form: &Self::UpdateForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(address::table.find(id))
     .set(form)
     .get_result::<Self>(conn)
     .await
     .with_fastjob_type(FastJobErrorType::CouldntDeleteAddress)
  }
}