use crate::{
  newtypes::IdentityCardId,
  source::identity_card::{IdentityCard, IdentityCardInsertForm, IdentityCardUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::dsl::{exists, not};
use diesel::{dsl::insert_into, select, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::identity_card;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Crud for IdentityCard {
  type InsertForm = IdentityCardInsertForm;
  type UpdateForm = IdentityCardUpdateForm;
  type IdType = IdentityCardId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(identity_card::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateIdentityCard)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(identity_card::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateIdentityCard)
  }

  async fn delete(pool: &mut DbPool<'_>, id: Self::IdType) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(identity_card::table.find(id))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntDeleteIdentityCard)
  }
}

impl IdentityCard {
  pub async fn check_id_number_exist(pool: &mut DbPool<'_>, id: &IdentityCardId, id_number: &str) -> FastJobResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(not(exists(
      identity_card::table
        .filter(identity_card::id.ne(id))
        .filter(identity_card::id_number.eq(id_number))
        .filter(identity_card::is_verified.eq(true)),
    )))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(FastJobErrorType::IDNumberAlreadyExist.into())
  }
}
