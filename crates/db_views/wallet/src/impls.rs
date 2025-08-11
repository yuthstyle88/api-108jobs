use crate::WalletView;
use diesel::{prelude::*, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{LocalUserId, WalletId},
  source::{
    local_user::LocalUser,
    wallet::{Wallet, WalletInsertForm, WalletUpdateForm},
  },
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::{local_user, wallet};
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl WalletView {
  #[inline]
  fn validate_positive_amount(amount: f64) -> FastJobResult<()> {
    if amount <= 0.0 {
      return Err(FastJobErrorType::InvalidField("Amount must be positive".to_string()))?;
    }
    Ok(())
  }

  pub async fn read(pool: &mut DbPool<'_>, wallet_id: WalletId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let wallet = wallet::table.find(wallet_id).first::<Wallet>(conn).await?;
    Ok(WalletView { wallet })
  }

  pub async fn read_by_user(pool: &mut DbPool<'_>, user_id: LocalUserId) -> FastJobResult<Wallet> {
    let conn = &mut get_conn(pool).await?;
    
    let wallet_opt = local_user::table
      .inner_join(wallet::table.on(local_user::wallet_id.eq(wallet::id.nullable())))
      .filter(local_user::id.eq(user_id))
      .select(Wallet::as_select())
      .first::<Wallet>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntFindWalletByUser)?;
    Ok(wallet_opt)
  }
  pub async fn create_wallet(pool: &mut DbPool<'_>, form: &WalletInsertForm) -> Result<Wallet, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(wallet::table)
      .values(form)
      .get_result::<Wallet>(conn)
      .await
  }

  pub async fn update_wallet(
    pool: &mut DbPool<'_>,
    wallet_id: WalletId,
    form: &WalletUpdateForm,
  ) -> Result<Wallet, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(wallet::table.find(wallet_id))
      .set(form)
      .get_result::<Wallet>(conn)
      .await
  }

  pub async fn create_for_user(pool: &mut DbPool<'_>, user_id: LocalUserId) -> FastJobResult<Wallet> {
    let conn = &mut get_conn(pool).await?;
    
    // Check if user already has a wallet
    let existing_wallet = local_user::table
      .filter(local_user::id.eq(user_id))
      .filter(local_user::wallet_id.is_not_null())
      .first::<LocalUser>(conn)
      .await
      .optional()?;

    if existing_wallet.is_some() {
      return Err(FastJobErrorType::WalletAlreadyExists)?;
    }

    // Create wallet
    let wallet_form = WalletInsertForm {
      balance: Some(0.0),
      escrow_balance: Some(0.0),
      created_at: None,
    };
    
    let wallet = diesel::insert_into(wallet::table)
      .values(&wallet_form)
      .get_result::<Wallet>(conn)
      .await?;

    // Update user with wallet_id
    diesel::update(local_user::table.find(user_id))
      .set(local_user::wallet_id.eq(wallet.id))
      .execute(conn)
      .await?;

    Ok(wallet)
  }


  /// Deposit funds into user's wallet
  pub async fn deposit_funds(
    pool: &mut DbPool<'_>,
    wallet_id: WalletId,
    amount: f64,
  ) -> FastJobResult<Wallet> {
    Self::validate_positive_amount(amount)?;

    Wallet::update_balance(pool, wallet_id, amount).await
  }

  /// Pay for a job (deduct from client's wallet and hold in escrow)
  pub async fn pay_for_job(
    pool: &mut DbPool<'_>,
    wallet_id: WalletId,
    amount: f64,
  ) -> FastJobResult<Wallet> {
    Self::validate_positive_amount(amount)?;
    
    // Deduct from client's wallet (negative amount)
    Wallet::update_balance(pool, wallet_id, -amount).await
  }

  /// Complete job payment (transfer from escrow to freelancer)
  pub async fn complete_job_payment(
    pool: &mut DbPool<'_>,
    freelancer_user_id: LocalUserId,
    amount: f64,
  ) -> FastJobResult<Wallet> {
    Self::validate_positive_amount(amount)?;
    
    // Create wallet for freelancer if it doesn't exist
    let wallet_view = Self::read_by_user(pool, freelancer_user_id).await?;
    Self::create_for_user(pool, freelancer_user_id).await?;
    let wallet_id = wallet_view.id;
    // Add to freelancer's wallet
    Wallet::update_balance(pool, wallet_id, amount).await
  }

  /// Transfer funds between two users
  pub async fn transfer_funds(
    pool: &mut DbPool<'_>,
    from_wallet_id: WalletId,
    to_wallet_id: WalletId,
    amount: f64,
  ) -> FastJobResult<(Wallet, Wallet)> {
    Self::validate_positive_amount(amount)?;
    // Deduct from sender
    let from_wallet = Wallet::update_balance(pool, from_wallet_id, -amount).await?;
    
    // Add to receiver
    let to_wallet = Wallet::update_balance(pool, to_wallet_id, amount).await?;
    
    Ok((from_wallet, to_wallet))
  }

  /// Admin operation: Top up user wallet
  pub async fn admin_top_up(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    amount: f64,
  ) -> FastJobResult<(Wallet, f64)> {
    Self::validate_positive_amount(amount)?;

    // Get current balance before operation
    let current_wallet = Self::read_by_user(pool, user_id).await?;
    let current_balance = current_wallet.balance;

    // Create wallet if doesn't exist
    Self::create_for_user(pool, user_id).await?;
    let wallet_id = current_wallet.id;
    // Add funds to wallet
    let updated_wallet = Wallet::update_balance(pool, wallet_id, amount).await?;
    
    Ok((updated_wallet, current_balance))
  }

  /// Admin operation: Withdraw from user wallet
  pub async fn admin_withdraw(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    amount: f64,
  ) -> FastJobResult<(Wallet, f64)> {
    Self::validate_positive_amount(amount)?;

    // Get current wallet and check balance
    let current_wallet = Self::read_by_user(pool, user_id).await?;
    
    let current_balance = current_wallet.balance;
    
    if current_balance < amount {
      return Err(FastJobErrorType::InsufficientBalance)?;
    }
    let wallet_id = current_wallet.id;
    // Deduct funds from wallet (negative amount)
    let updated_wallet = Wallet::update_balance(pool, wallet_id, -amount).await?;
    
    Ok((updated_wallet, current_balance))
  }
}