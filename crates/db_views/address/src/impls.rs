use crate::AddressView;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  utils::{get_conn, DbPool},
};
use lemmy_db_schema::newtypes::AddressId;
use lemmy_db_schema_file::schema::address;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl AddressView {
  pub async fn find_by_id(
    pool: &mut DbPool<'_>,
    address_id: AddressId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    address::table
      .filter(address::id.eq(address_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntFindAddress)
  }
}
