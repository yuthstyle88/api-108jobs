use crate::{
  diesel::{ExpressionMethods, QueryDsl},
  newtypes::LocalUserId,
  source::login_token::{LoginToken, LoginTokenCreateForm},
  utils::{get_conn, DbPool},
};
use diesel::{delete, dsl::exists, insert_into, select};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::login_token::{dsl::login_token, user_id};
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl LoginToken {
  pub async fn create(pool: &mut DbPool<'_>, form: LoginTokenCreateForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(login_token)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateLoginToken)
  }

  /// Check if the given token is valid for user.
  pub async fn validate(
    pool: &mut DbPool<'_>,
    user_id_: LocalUserId,
    token_: &str,
  ) -> FastJobResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      login_token.find(token_).filter(user_id.eq(user_id_)),
    ))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(FastJobErrorType::NotLoggedIn.into())
  }

  pub async fn list(pool: &mut DbPool<'_>, user_id_: LocalUserId) -> FastJobResult<Vec<LoginToken>> {
    let conn = &mut get_conn(pool).await?;

    login_token
      .filter(user_id.eq(user_id_))
      .get_results(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  /// Invalidate specific token on user logout.
  pub async fn invalidate(pool: &mut DbPool<'_>, token_: &str) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    delete(login_token.find(token_))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::Deleted)
  }

  /// Invalidate all logins of given user on password reset/change, or account deletion.
  pub async fn invalidate_all(pool: &mut DbPool<'_>, user_id_: LocalUserId) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    delete(login_token.filter(user_id.eq(user_id_)))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::Deleted)
  }
}
