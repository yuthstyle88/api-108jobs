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
use lemmy_utils::error::{FastJobErrorType, FastJobResult};

impl WalletView {
  pub async fn read(pool: &mut DbPool<'_>, wallet_id: WalletId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let wallet = wallet::table.find(wallet_id).first::<Wallet>(conn).await?;
    Ok(WalletView { wallet })
  }

  pub async fn read_by_user(pool: &mut DbPool<'_>, user_id: LocalUserId) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    
    let wallet_opt = local_user::table
      .inner_join(wallet::table.on(local_user::wallet_id.eq(wallet::id.nullable())))
      .filter(local_user::id.eq(user_id))
      .select(Wallet::as_select())
      .first::<Wallet>(conn)
      .await
      .optional()?;

    Ok(wallet_opt.map(|wallet| WalletView { wallet }))
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

  pub async fn update_balance(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    amount: f64,
  ) -> FastJobResult<Wallet> {
    let conn = &mut get_conn(pool).await?;
    
    // Get user's wallet
    let user = local_user::table
      .filter(local_user::id.eq(user_id))
      .first::<LocalUser>(conn)
      .await?;

    let wallet_id = user.wallet_id.ok_or(FastJobErrorType::WalletNotFound)?;

    // Get current wallet
    let current_wallet = wallet::table
      .find(wallet_id)
      .first::<Wallet>(conn)
      .await?;

    let new_balance = current_wallet.balance.unwrap_or(0.0) + amount;
    
    if new_balance < 0.0 {
      return Err(FastJobErrorType::InsufficientBalance)?;
    }

    // Update wallet
    let update_form = WalletUpdateForm {
      balance: Some(new_balance),
      escrow_balance: None, // Don't change escrow balance in regular deposits
      updated_at: Some(chrono::Utc::now()),
    };

    diesel::update(wallet::table.find(wallet_id))
      .set(&update_form)
      .get_result::<Wallet>(conn)
      .await
      .map_err(Into::into)
  }

  /// Deposit funds into user's wallet
  pub async fn deposit_funds(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    amount: f64,
  ) -> FastJobResult<Wallet> {
    if amount <= 0.0 {
      return Err(FastJobErrorType::InvalidField("Amount must be positive".to_string()))?;
    }
    
    Self::update_balance(pool, user_id, amount).await
  }

  /// Pay for a job (deduct from client's wallet and hold in escrow)
  pub async fn pay_for_job(
    pool: &mut DbPool<'_>,
    client_user_id: LocalUserId,
    amount: f64,
  ) -> FastJobResult<Wallet> {
    if amount <= 0.0 {
      return Err(FastJobErrorType::InvalidField("Amount must be positive".to_string()))?;
    }
    
    // Deduct from client's wallet (negative amount)
    Self::update_balance(pool, client_user_id, -amount).await
  }

  /// Complete job payment (transfer from escrow to freelancer)
  pub async fn complete_job_payment(
    pool: &mut DbPool<'_>,
    freelancer_user_id: LocalUserId,
    amount: f64,
  ) -> FastJobResult<Wallet> {
    if amount <= 0.0 {
      return Err(FastJobErrorType::InvalidField("Amount must be positive".to_string()))?;
    }
    
    // Create wallet for freelancer if it doesn't exist
    let wallet_view = Self::read_by_user(pool, freelancer_user_id).await?;
    if wallet_view.is_none() {
      Self::create_for_user(pool, freelancer_user_id).await?;
    }
    
    // Add to freelancer's wallet
    Self::update_balance(pool, freelancer_user_id, amount).await
  }

  /// Transfer funds between two users
  pub async fn transfer_funds(
    pool: &mut DbPool<'_>,
    from_user_id: LocalUserId,
    to_user_id: LocalUserId,
    amount: f64,
  ) -> FastJobResult<(Wallet, Wallet)> {
    if amount <= 0.0 {
      return Err(FastJobErrorType::InvalidField("Amount must be positive".to_string()))?;
    }

    // Deduct from sender
    let from_wallet = Self::update_balance(pool, from_user_id, -amount).await?;
    
    // Add to receiver
    let to_wallet = Self::update_balance(pool, to_user_id, amount).await?;
    
    Ok((from_wallet, to_wallet))
  }

  /// Admin operation: Top up user wallet
  pub async fn admin_top_up(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    amount: f64,
  ) -> FastJobResult<(Wallet, f64)> {
    if amount <= 0.0 {
      return Err(FastJobErrorType::InvalidField("Amount must be positive".to_string()))?;
    }

    // Get current balance before operation
    let current_wallet = Self::read_by_user(pool, user_id).await?;
    let previous_balance = current_wallet.as_ref()
      .map(|w| w.wallet.balance.unwrap_or(0.0))
      .unwrap_or(0.0);

    // Create wallet if doesn't exist
    if current_wallet.is_none() {
      Self::create_for_user(pool, user_id).await?;
    }

    // Add funds to wallet
    let updated_wallet = Self::update_balance(pool, user_id, amount).await?;
    
    Ok((updated_wallet, previous_balance))
  }

  /// Admin operation: Withdraw from user wallet
  pub async fn admin_withdraw(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    amount: f64,
  ) -> FastJobResult<(Wallet, f64)> {
    if amount <= 0.0 {
      return Err(FastJobErrorType::InvalidField("Amount must be positive".to_string()))?;
    }

    // Get current wallet and check balance
    let current_wallet = Self::read_by_user(pool, user_id).await?
      .ok_or(FastJobErrorType::WalletNotFound)?;
    
    let previous_balance = current_wallet.wallet.balance.unwrap_or(0.0);
    
    if previous_balance < amount {
      return Err(FastJobErrorType::InsufficientBalance)?;
    }

    // Deduct funds from wallet (negative amount)
    let updated_wallet = Self::update_balance(pool, user_id, -amount).await?;
    
    Ok((updated_wallet, previous_balance))
  }
}