use crate::WalletView;
use diesel::{result::Error, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{LocalUserId, WalletId},
  source::wallet::Wallet,
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::wallet;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};

impl WalletView {

  pub async fn read(pool: &mut DbPool<'_>, wallet_id: WalletId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let wallet = wallet::table.find(wallet_id).first::<Wallet>(conn).await?;
    Ok(WalletView { wallet })
  }

  pub async fn read_by_user(pool: &mut DbPool<'_>, user_id: LocalUserId) -> FastJobResult<Wallet> {
    Wallet::get_by_user(pool, user_id).await
  }

}