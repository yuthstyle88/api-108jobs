use crate::IdentityCardView;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{IdentityCardId, LocalUserId},
  source::identity_card::IdentityCard,
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::identity_card;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl IdentityCardView {
  pub async fn find_by_local_user_id(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    identity_card::table
      .filter(identity_card::local_user_id.eq(local_user_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntFindIdentityCard)
  }
}
