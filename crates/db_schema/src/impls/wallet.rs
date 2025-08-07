use crate::{
  newtypes::{LocalUserId, WalletId},
  source::{
    local_user::LocalUser,
    wallet::{Wallet, WalletInsertForm, WalletUpdateForm},
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};

use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, NullableExpressionMethods};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{local_user, wallet};
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use chrono::Utc;

impl Wallet {
  /// Get a wallet by user ID
  pub async fn get_by_user(pool: &mut DbPool<'_>, user_id: LocalUserId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    
    let wallet = local_user::table
      .inner_join(wallet::table.on(local_user::wallet_id.eq(wallet::id.nullable())))
      .filter(local_user::id.eq(user_id))
     .select(wallet::all_columns)
      .first::<Wallet>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntFindWalletByUser)?;
    
    Ok(wallet)
  }

  /// Create a wallet for a user
  pub async fn create_for_user(pool: &mut DbPool<'_>, user_id: LocalUserId) -> FastJobResult<Self> {
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
      created_at: Some(Utc::now()),
    };
    
    let wallet = diesel::insert_into(wallet::table)
      .values(&wallet_form)
      .get_result::<Self>(conn)
      .await?;

    // Update user with wallet_id
    diesel::update(local_user::table.find(user_id))
      .set(local_user::wallet_id.eq(wallet.id))
      .execute(conn)
      .await?;

    Ok(wallet)
  }

  /// Update wallet balance
  pub async fn update_balance(
      pool: &mut DbPool<'_>,
      wallet_id: WalletId,
      amount: f64,
  ) -> FastJobResult<Self> {
      if amount == 0.0 {
          return Err(FastJobErrorType::InvalidField("Amount cannot be zero".into()))?;
      }

      let conn = &mut get_conn(pool).await?;

      let wallet = wallet::table
          .find(wallet_id)
          .first::<Self>(conn)
          .await
          .map_err(|_| FastJobErrorType::WalletNotFound)?;

      let new_balance = wallet.balance + amount;

      let update_form = WalletUpdateForm {
          balance: Some(new_balance),
          escrow_balance: None,
          updated_at: Some(Utc::now()),
      };

      diesel::update(wallet::table.find(wallet_id))
          .set(&update_form)
          .get_result::<Self>(conn)
          .await
          .map_err(Into::into)
  }

  /// Update escrow balance
  pub async fn update_escrow_balance(
    pool: &mut DbPool<'_>,
    wallet_id: WalletId,
    amount: f64,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    
    // Get current wallet
    let current_wallet = wallet::table
      .find(wallet_id)
      .first::<Self>(conn)
      .await?;

    let new_escrow_balance = current_wallet.escrow_balance + amount;

    // Update wallet
    let update_form = WalletUpdateForm {
      balance: None,
      escrow_balance: Some(new_escrow_balance),
      updated_at: Some(Utc::now()),
    };

    diesel::update(wallet::table.find(wallet_id))
      .set(&update_form)
      .get_result::<Self>(conn)
      .await
      .map_err(Into::into)
  }

  /// Transfer from balance to escrow
  pub async fn transfer_to_escrow(
    pool: &mut DbPool<'_>,
    wallet_id: WalletId,
    amount: f64,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    
    // Get current wallet
    let current_wallet = wallet::table
      .find(wallet_id)
      .first::<Self>(conn)
      .await?;

    if current_wallet.balance < amount {
      return Err(FastJobErrorType::InsufficientBalance)?;
    }

    let new_balance = current_wallet.balance - amount;
    let new_escrow_balance = current_wallet.escrow_balance + amount;

    // Update wallet
    let update_form = WalletUpdateForm {
      balance: Some(new_balance),
      escrow_balance: Some(new_escrow_balance),
      updated_at: Some(Utc::now()),
    };

    diesel::update(wallet::table.find(wallet_id))
      .set(&update_form)
      .get_result::<Self>(conn)
      .await
      .map_err(Into::into)
  }

  /// Transfer from escrow to balance
  pub async fn transfer_from_escrow(
    pool: &mut DbPool<'_>,
    wallet_id: WalletId,
    amount: f64,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    
    // Get current wallet
    let current_wallet = wallet::table
      .find(wallet_id)
      .first::<Self>(conn)
      .await?;

    if current_wallet.escrow_balance < amount {
      return Err(FastJobErrorType::InsufficientEscrowBalance)?;
    }

    let new_balance = current_wallet.balance + amount;
    let new_escrow_balance = current_wallet.escrow_balance - amount;

    // Update wallet
    let update_form = WalletUpdateForm {
      balance: Some(new_balance),
      escrow_balance: Some(new_escrow_balance),
      updated_at: Some(Utc::now()),
    };

    diesel::update(wallet::table.find(wallet_id))
      .set(&update_form)
      .get_result::<Self>(conn)
      .await
      .map_err(Into::into)
  }
}

impl Crud for Wallet {
  type InsertForm = WalletInsertForm;
  type UpdateForm = WalletUpdateForm;
  type IdType = WalletId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(wallet::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateWallet)
  }

  async fn update(pool: &mut DbPool<'_>, id: WalletId, form: &Self::UpdateForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(wallet::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateWallet)
  }
}