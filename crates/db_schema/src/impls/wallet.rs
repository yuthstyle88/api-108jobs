/// Move funds from one wallet to another using the three-balance model.
use crate::{
  newtypes::{LocalUserId, WalletId},
  source::wallet::{Wallet, WalletInsertForm, WalletUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};

use crate::source::wallet::{
  TxKind, WalletTransaction, WalletTransactionInsertForm, WalletTransactionUpdateForm,
};
use chrono::Utc;
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{local_user, wallet, wallet_transaction};
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

enum WalletOp {
  Deposit,
  Hold,
  Release,
  Spend,
  TransferOut, // available -= amount; total -= amount (direct transfer out)
  TransferIn,  // available += amount; total += amount (direct transfer in)
}
impl Wallet {
  /// Load a wallet row with FOR UPDATE lock
  async fn load_for_update(
    conn: &mut diesel_async::AsyncPgConnection,
    id: WalletId,
  ) -> FastJobResult<Self> {
    let w = wallet::table
      .find(id)
      .for_update()
      .first::<Self>(conn)
      .await?;
    Ok(w)
  }

  /// Compute new balances for an operation. Returns (total, available, outstanding)
  fn compute_new_balances(w: &Self, op: &WalletOp, amount: f64) -> FastJobResult<(f64, f64, f64)> {
    if amount <= 0.0 {
      return Err(FastJobErrorType::InvalidField("Amount must be positive".into()).into());
    }
    let (mut t, mut a, mut o) = (w.balance_total, w.balance_available, w.balance_outstanding);
    match op {
      WalletOp::Deposit => {
        t += amount;
        a += amount;
      }
      WalletOp::Hold => {
        if a < amount {
          return Err(
            FastJobErrorType::InvalidField("Insufficient available balance".into()).into(),
          );
        }
        a -= amount;
        o += amount;
      }
      WalletOp::Release => {
        if o < amount {
          return Err(
            FastJobErrorType::InvalidField("Insufficient outstanding balance".into()).into(),
          );
        }
        a += amount;
        o -= amount;
      }
      WalletOp::Spend => {
        if o < amount {
          return Err(
            FastJobErrorType::InvalidField("Insufficient outstanding balance".into()).into(),
          );
        }
        if t < amount {
          return Err(FastJobErrorType::InvalidField("Insufficient total balance".into()).into());
        }
        t -= amount;
        o -= amount;
      }
      WalletOp::TransferOut => {
        if a < amount || t < amount {
          return Err(FastJobErrorType::InvalidField("Insufficient balance for transfer".into()).into());
        }
        a -= amount;
        t -= amount;
      }
      WalletOp::TransferIn => {
        a += amount;
        t += amount;
      }
    }
    // invariants
    if a < 0.0 || o < 0.0 || (a + o - t).abs() > 1e-9 {
      return Err(FastJobErrorType::InvalidField("Wallet invariant violated".into()).into());
    }
    Ok((t, a, o))
  }

  /// Apply an operation using WalletUpdateForm as the single source of truth
  async fn apply_op_on(
    conn: &mut diesel_async::AsyncPgConnection,
    id: WalletId,
    op: WalletOp,
    amount: f64,
  ) -> FastJobResult<Self> {
    let current = Self::load_for_update(conn, id).await?;
    let (t, a, o) = Self::compute_new_balances(&current, &op, amount)?;
    let form = WalletUpdateForm {
      balance_total: Some(t),
      balance_available: Some(a),
      balance_outstanding: Some(o),
      is_platform: None,
      updated_at: Some(Utc::now()),
    };
    let w = diesel::update(wallet::table.find(id))
      .set(&form)
      .get_result::<Self>(conn)
      .await?;
    Ok(w)
  }

  #[inline]
  fn validate_positive_amount(amount: f64) -> FastJobResult<()> {
    if amount <= 0.0 {
      return Err(FastJobErrorType::InvalidField("Amount must be positive".to_string()).into());
    }
    Ok(())
  }

  /// Get a wallet by user ID
  pub async fn get_by_user(pool: &mut DbPool<'_>, user_id: LocalUserId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    let wallet = local_user::table
      .inner_join(wallet::table.on(local_user::wallet_id.eq(wallet::id)))
      .filter(local_user::id.eq(user_id))
      .select(wallet::all_columns)
      .first::<Wallet>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntFindWalletByUser)?;

    Ok(wallet)
  }

  /// Create a wallet transaction for deposit or withdraw.
  /// For transfers, this facade cannot be used because two mirrored forms are required.
  pub async fn create_transaction(
    pool: &mut DbPool<'_>,
    form: &WalletTransactionInsertForm,
  ) -> FastJobResult<Wallet> {
    // Façade kept for compatibility; forwards to appropriate handler and returns the updated user wallet.
    let w = match form.kind {
      TxKind::Deposit => {
        Self::deposit_from_platform(pool, form).await?
      }
      TxKind::Withdraw => {
        Self::withdraw_to_platform(pool, form).await?
      }
      TxKind::Transfer => {
        return Err(FastJobErrorType::InvalidField(
          "Transfer requires two forms; use transfer_between_wallets with outgoing and incoming entries sharing the same idempotency_key".into(),
        ).into());
      }
    };
    Ok(w)
  }

  /// Ensures a single platform wallet exists, operating on the provided transaction connection
  /// Internal helper: create an empty wallet (platform or user) on the same connection/txn.
  async fn create_blank_on(
    conn: &mut diesel_async::AsyncPgConnection,
    is_platform: bool,
  ) -> FastJobResult<Self> {
    let form = WalletInsertForm {
      balance_total: Some(0.0),
      balance_available: Some(0.0),
      balance_outstanding: Some(0.0),
      is_platform: Some(is_platform),
      created_at: Some(Utc::now()),
    };
    let w = diesel::insert_into(wallet::table)
    .values(&form)
    .get_result::<Self>(conn)
    .await
    .with_fastjob_type(FastJobErrorType::CouldntCreateWallet)?;
    Ok(w)
  }

  /// Create a wallet for a user (non-platform) on the given connection (caller links to local_user).
  pub async fn create_for_user(
    conn: &mut diesel_async::AsyncPgConnection,
  ) -> FastJobResult<Self> {
    Self::create_blank_on(conn, false).await
  }

  /// Get the platform wallet id (must be pre-seeded). Error if missing.
  async fn platform_wallet_id(
    conn: &mut diesel_async::AsyncPgConnection,
  ) -> FastJobResult<WalletId> {
    let id = wallet::table
    .filter(wallet::is_platform.eq(true))
    .select(wallet::id)
    .first::<WalletId>(conn)
    .await
    .optional()?;
    match id {
      Some(wid) => Ok(wid),
      None => Err(FastJobErrorType::InvalidField("Platform wallet not initialized".into()).into()),
    }
  }

  /// Deposit funds: total += amount, available += amount
  pub async fn deposit(
    pool: &mut DbPool<'_>,
    wallet_id: WalletId,
    amount: f64,
  ) -> FastJobResult<Self> {
    Self::validate_positive_amount(amount)?;
    let conn = &mut get_conn(pool).await?;
    let updated = Self::apply_op_on(conn, wallet_id, WalletOp::Deposit, amount).await?;
    Ok(updated)
  }

  /// Hold funds: available -= amount, outstanding += amount (guard: available >= amount)
  pub async fn hold(
    pool: &mut DbPool<'_>,
    wallet_id: WalletId,
    amount: f64,
  ) -> FastJobResult<Self> {
    Self::validate_positive_amount(amount)?;
    let conn = &mut get_conn(pool).await?;
    let updated = Self::apply_op_on(conn, wallet_id, WalletOp::Hold, amount).await?;
    Ok(updated)
  }

  /// Release held funds: available += amount, outstanding -= amount (guard: outstanding >= amount)
  pub async fn release(
    pool: &mut DbPool<'_>,
    wallet_id: WalletId,
    amount: f64,
  ) -> FastJobResult<Self> {
    Self::validate_positive_amount(amount)?;
    let conn = &mut get_conn(pool).await?;
    let updated = Self::apply_op_on(conn, wallet_id, WalletOp::Release, amount).await?;
    Ok(updated)
  }

  /// Spend finalized from outstanding: total -= amount, outstanding -= amount (guards: outstanding >= amount, total >= amount)
  pub async fn spend(
    pool: &mut DbPool<'_>,
    wallet_id: WalletId,
    amount: f64,
  ) -> FastJobResult<Self> {
    Self::validate_positive_amount(amount)?;
    let conn = &mut get_conn(pool).await?;
    let updated = Self::apply_op_on(conn, wallet_id, WalletOp::Spend, amount).await?;
    Ok(updated)
  }
  /// Insert a wallet transaction row, pair-friendly, reference-based.
  async fn insert_wallet_tx(
    conn: &mut diesel_async::AsyncPgConnection,
    form: &WalletTransactionInsertForm,
  ) -> FastJobResult<WalletTransaction> {
    let tx = diesel::insert_into(wallet_transaction::table)
      .values(form)
      .get_result::<WalletTransaction>(conn)
      .await?;
    Ok(tx)
  }

  /// Paired transfer: from -> to. Always creates two wallet_transaction rows with the same idempotency_key.
  /// Paired transfer: caller must pass two forms (outgoing + incoming) using the same idempotency_key.
  pub async fn transfer_between_wallets(
    pool: &mut DbPool<'_>,
    form_out: &WalletTransactionInsertForm, // from -> ...
    form_in:  &WalletTransactionInsertForm, // ... -> to
  ) -> FastJobResult<()> {
    Self::validate_positive_amount(form_out.amount)?;
    if (form_out.amount - form_in.amount).abs() > 1e-9 {
      return Err(FastJobErrorType::InvalidField("Mismatched amounts for transfer pair".into()).into());
    }
    if form_out.idempotency_key != form_in.idempotency_key {
      return Err(FastJobErrorType::InvalidField("Idempotency key must match for transfer pair".into()).into());
    }
    let from   = form_out.wallet_id;
    let to     = form_in.wallet_id;
    let amount = form_out.amount;

    let conn = &mut get_conn(pool).await?;
    conn.run_transaction(|conn| {
      async move {
        Self::move_funds(conn, from, to, amount).await?;
        let _ = Self::insert_wallet_tx(conn, form_out).await?;
        let _ = Self::insert_wallet_tx(conn, form_in).await?;
        Ok::<_, lemmy_utils::error::FastJobError>(())
      }.scope_boxed()
    }).await?;
    Ok(())
  }

  /// Deposit from platform -> user: must create a pair of journal rows
  pub async fn deposit_from_platform(
    pool: &mut DbPool<'_>,
    form: &WalletTransactionInsertForm,
  ) -> FastJobResult<Wallet> {
    let amount = form.amount;
    Self::validate_positive_amount(amount)?;

    let conn = &mut get_conn(pool).await?;
    return conn
      .run_transaction(|conn| {
        async move {
          // move funds: platform -> user
          let platform_id = Self::platform_wallet_id(conn).await?;
          Self::move_funds(conn, platform_id, form.wallet_id, amount).await?;
          // journal user side
          let _ = Self::insert_wallet_tx(conn, form).await?;
          // mirrored platform-side entry
          let mut mirror = form.clone();
          mirror.wallet_id = platform_id;
          mirror.description = format!("platform counter: {}", mirror.description);
          let _ = Self::insert_wallet_tx(conn, &mirror).await?;
          // return updated user wallet
          let w = Self::load_for_update(conn, form.wallet_id).await?;
          Ok::<_, lemmy_utils::error::FastJobError>(w)
        }
        .scope_boxed()
      })
      .await;
  }

  /// Withdraw to platform (user -> platform): must create a pair of journal rows
  pub async fn withdraw_to_platform(
    pool: &mut DbPool<'_>,
    form: &WalletTransactionInsertForm,
  ) -> FastJobResult<Wallet> {
    let amount = form.amount;
    Self::validate_positive_amount(amount)?;

    let conn = &mut get_conn(pool).await?;
    return conn
      .run_transaction(|conn| {
        async move {
          // move funds: user -> platform
          let platform_id = Self::platform_wallet_id(conn).await?;
          Self::move_funds(conn, form.wallet_id, platform_id, amount).await?;
          // journal user side
          let _ = Self::insert_wallet_tx(conn, form).await?;
          // mirrored platform-side entry
          let mut mirror = form.clone();
          mirror.wallet_id = platform_id;
          mirror.description = format!("platform counter: {}", mirror.description);
          let _ = Self::insert_wallet_tx(conn, &mirror).await?;
          // return updated user wallet
          let w = Self::load_for_update(conn, form.wallet_id).await?;
          Ok::<_, lemmy_utils::error::FastJobError>(w)
        }
        .scope_boxed()
      })
      .await;
  }
  async fn move_funds(
    conn: &mut diesel_async::AsyncPgConnection,
    from_wallet: WalletId,
    to_wallet: WalletId,
    amount: f64,
  ) -> FastJobResult<()> {
    // basic validation
    Self::validate_positive_amount(amount)?;
    if from_wallet == to_wallet {
      return Err(FastJobErrorType::InvalidField("Cannot transfer to the same wallet".into()).into());
    }

    // Direct transfer: decrease from.available & from.total; increase to.available & to.total
    let _ = Self::apply_op_on(conn, from_wallet, WalletOp::TransferOut, amount).await?;
    let _ = Self::apply_op_on(conn, to_wallet,   WalletOp::TransferIn,  amount).await?;
    Ok(())
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

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(wallet::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateWallet)
  }
}

impl Crud for WalletTransaction {
  type InsertForm = WalletTransactionInsertForm;
  type UpdateForm = WalletTransactionUpdateForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(wallet_transaction::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateWalletTranSaction)
  }

  async fn update(pool: &mut DbPool<'_>, id: i32, form: &Self::UpdateForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(wallet_transaction::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateWalletTranSaction)
  }
}
