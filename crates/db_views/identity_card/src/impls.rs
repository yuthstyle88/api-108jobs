use crate::IdentityCardView;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  utils::{get_conn, DbPool},
};
use lemmy_db_schema::newtypes::IdentityCardId;
use lemmy_db_schema_file::schema::identity_card;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl IdentityCardView {
  pub async fn find_by_id(
    pool: &mut DbPool<'_>,
    identity_card_id: IdentityCardId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    identity_card::table
      .filter(identity_card::id.eq(identity_card_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntFindIdentityCard)
  }
}
