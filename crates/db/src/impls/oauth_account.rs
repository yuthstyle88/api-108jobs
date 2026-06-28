use crate::{
  newtypes::LocalUserId,
  schema::{oauth_account, oauth_account::dsl::local_user_id},
  source::oauth_account::{OAuthAccount, OAuthAccountInsertForm},
  utils::{get_conn, DbPool},
};
use app_108jobs_core::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use diesel::{insert_into, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

impl OAuthAccount {
  pub async fn create(pool: &mut DbPool<'_>, form: &OAuthAccountInsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(oauth_account::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateOauthAccount)
  }

  pub async fn delete_user_accounts(
    pool: &mut DbPool<'_>,
    for_local_user_id: LocalUserId,
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;

    diesel::delete(oauth_account::table.filter(local_user_id.eq(for_local_user_id)))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::Deleted)
  }
}
