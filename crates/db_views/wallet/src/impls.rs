use crate::WalletView;
use diesel::{result::Error, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::newtypes::LocalUserId;
use lemmy_db_schema::source::wallet::Wallet;
use lemmy_db_schema::{
  newtypes::WalletId,
  source::wallet::WalletModel,
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::wallet;
use lemmy_utils::error::FastJobResult;

impl WalletView {

  pub async fn read(pool: &mut DbPool<'_>, wallet_id: WalletId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let wallet = wallet::table.find(wallet_id).first::<Wallet>(conn).await?;
    Ok(WalletView { wallet })
  }

  pub async fn read_by_user(pool: &mut DbPool<'_>, local_user_id: LocalUserId) -> FastJobResult<Wallet> {
    WalletModel::get_by_user(pool, local_user_id).await
  }

}