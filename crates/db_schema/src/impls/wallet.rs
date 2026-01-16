/// Move funds from one wallet to another using the three-balance model.
use crate::{
  newtypes::WalletId,
  source::wallet::{WalletInsertForm, WalletModel, WalletUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};

use crate::newtypes::{Coin, CoinId, LocalUserId};
use crate::source::coin::CoinModel;
use crate::source::wallet::{TxKind, Wallet, WalletTransaction, WalletTransactionInsertForm, WalletTransactionUpdateForm};
use chrono::Utc;
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::RunQueryDsl;
use app_108jobs_db_schema_file::schema::{person,local_user, wallet, wallet_transaction};
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};


/// Internal enum for modeling balance mutations only (not transaction kinds).
enum BalanceOp {
  TransferOut, // available -= amount; total -= amount (direct transfer out)
  TransferIn,  // available += amount; total += amount (direct transfer in)
  Reserve,     // available -= amount; outstanding += amount
  Release,     // outstanding -= amount; available += amount
  Capture,     // outstanding -= amount; total -= amount
}

impl WalletModel {
  pub fn new(platform_wallet_id: WalletId, balance_total: Coin,
             balance_available: Coin, balance_outstanding: Coin) -> Self {
    Self {
      platform_wallet_id,
      balance_total,
      balance_available,
      balance_outstanding,
    }
  }

  /// Load a wallet row with FOR UPDATE lock
  async fn load_for_update(
    conn: &mut diesel_async::AsyncPgConnection,
    id: WalletId,
  ) -> FastJobResult<Wallet> {
    let w = wallet::table
      .find(id)
      .for_update()
      .first::<Wallet>(conn)
      .await?;
    Ok(w)
  }

  /// Compute new balances for a balance operation. Returns (total, available, outstanding)
  fn compute_new_balances(w: &Wallet, op: &BalanceOp, amount: Coin) -> FastJobResult<(Coin, Coin, Coin)> {
    if amount <= 0 {
      return Err(FastJobErrorType::InvalidField("Amount must be positive".into()).into());
    }
    let (mut t, mut a, mut o) = (w.balance_total, w.balance_available, w.balance_outstanding);
    match op {
      BalanceOp::TransferOut => {
        // Direct transfer out to another wallet
        if a < amount || t < amount {
          return Err(FastJobErrorType::InsufficientBalanceForTransfer.into());
        }
        a -= amount;
        t -= amount;
        // outstanding unchanged
      }
      BalanceOp::TransferIn => {
        // Direct transfer in from another wallet
        a += amount;
        t += amount;
        // outstanding unchanged
      }
      BalanceOp::Reserve => {
        // Hold funds: available -= amount; outstanding += amount; total unchanged
        if a < amount {
          return Err(FastJobErrorType::InsufficientBalanceForTransfer.into());
        }
        a -= amount;
        o += amount;
      }
      BalanceOp::Release => {
        // Cancel hold: outstanding -= amount; available += amount; total unchanged
        if o < amount {
          return Err(FastJobErrorType::InvalidField("Release exceeds outstanding".into()).into());
        }
        o -= amount;
        a += amount;
      }
      BalanceOp::Capture => {
        // Finalize hold: outstanding -= amount; total -= amount; available unchanged
        if o < amount || t < amount {
          return Err(FastJobErrorType::InvalidField("Capture exceeds outstanding/total".into()).into());
        }
        o -= amount;
        t -= amount;
      }
    }
    // invariants
    if a < 0 || o < 0 || a + o != t {
      return Err(FastJobErrorType::WalletInvariantViolated.into());
    }
    Ok((t, a, o))
  }

  /// Apply a balance operation using WalletUpdateForm as the single source of truth
  async fn apply_op_on(
    conn: &mut diesel_async::AsyncPgConnection,
    id: WalletId,
    op: BalanceOp,
    amount: Coin,
  ) -> FastJobResult<Wallet> {
    // 1) Lock current row
    let current = Self::load_for_update(conn, id).await?;
    // 2) Compute next balances for this op
    let (t, a, o) = Self::compute_new_balances(&current, &op, amount)?;
    // 3) Persist
    let form = WalletUpdateForm {
      balance_total: Some(t),
      balance_available: Some(a),
      balance_outstanding: Some(o),
      is_platform: None,
      updated_at: Some(Utc::now()),
    };
    let w = diesel::update(wallet::table.find(id))
      .set(&form)
      .get_result::<Wallet>(conn)
      .await?;
    Ok(w)
  }

  #[inline]
  fn validate_positive_amount(amount:  Coin) -> FastJobResult<()> {
    if amount <= 0 {
      return Err(FastJobErrorType::InvalidField("Amount must be positive".to_string()).into());
    }
    Ok(())
  }

  /// Get a wallet by user ID
  pub async fn get_by_user(pool: &mut DbPool<'_>, local_user_id: LocalUserId) -> FastJobResult<Wallet> {
    let conn = &mut get_conn(pool).await?;

    let wallet = person::table
      .inner_join(wallet::table.on(person::wallet_id.eq(wallet::id)))
      .inner_join(local_user::table.on(person::id.eq(local_user::person_id)))
      .filter(local_user::id.eq(local_user_id))
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
    coin_id: CoinId,
    platform_wallet_id: WalletId,
  ) -> FastJobResult<Wallet> {
    // Façade kept for compatibility; forwards to appropriate handler and returns the updated user wallet.
    let w = match form.kind {
      TxKind::Deposit => {
        // Credit user wallet using platform escrow (mirrored journals)
        Self::deposit_from_platform(pool, form, coin_id, platform_wallet_id).await?
      }
      TxKind::Withdraw => {
        // Debit user wallet to platform escrow (mirrored journals)
        Self::withdraw_to_platform(pool, form, coin_id, platform_wallet_id).await?
      }
      TxKind::Transfer => {
        return Err(FastJobErrorType::InvalidField(
          "Transfer requires two forms; use transfer_between_wallets with outgoing and incoming entries sharing the same idempotency_key".into(),
        ).into());
      }
      TxKind::Reserve => {
        // Move available -> outstanding within the same wallet
        Self::reserve(pool, form).await?
      }
      TxKind::Release => {
        // Move outstanding -> available within the same wallet
        Self::release(pool, form).await?
      }
      TxKind::Capture => {
        // Finalize: outstanding -> settled (total decreases)
        Self::capture(pool, form).await?
      }
      TxKind::Refund => {
        // Refund back to user: treat as a credit from platform escrow
        Self::deposit_from_platform(pool, form, coin_id, platform_wallet_id).await?
      }
    };
    Ok(w)
  }

  /// Ensures a single platform wallet exists, operating on the provided transaction connection
  /// Internal helper: create an empty wallet (platform or user) on the same connection/txn.
  async fn create_blank_on(
    conn: &mut diesel_async::AsyncPgConnection,
    is_platform: bool,
  ) -> FastJobResult<Wallet> {
    let form = WalletInsertForm {
      balance_total: Some(Coin(0)),
      balance_available: Some(Coin(0)),
      balance_outstanding: Some(Coin(0)),
      is_platform: Some(is_platform),
      created_at: Some(Utc::now()),
    };
    let w = diesel::insert_into(wallet::table)
    .values(&form)
    .get_result::<Wallet>(conn)
    .await
    .with_fastjob_type(FastJobErrorType::CouldntCreateWallet)?;
    Ok(w)
  }

  /// Create a wallet for a user (non-platform) on the given connection (caller links to local_user).
  pub async fn create_for_user(
    conn: &mut diesel_async::AsyncPgConnection,
  ) -> FastJobResult<Wallet> {
    Self::create_blank_on(conn, false).await
  }

  pub async fn create_for_platform(
    conn: &mut diesel_async::AsyncPgConnection,
  ) -> FastJobResult<Wallet> {
    Self::create_blank_on(conn, true).await
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

  /// Deposit funds *from platform to user*.
  /// This delegates to `deposit_from_platform` so the platform wallet is debited
  /// and the user wallet is credited, with mirrored journal rows recorded.
  /// Callers must provide a `WalletTransactionInsertForm` with `kind = TxKind::Deposit`.
  pub async fn deposit(
      pool: &mut DbPool<'_>,
      form: &WalletTransactionInsertForm,
      coin_id: CoinId,
      platform_wallet_id: WalletId,
  ) -> FastJobResult<Wallet> {
      // Delegate to platform-backed deposit so coins are deducted from platform escrow.
      Self::deposit_from_platform(pool, form, coin_id, platform_wallet_id).await
  }

  /// Reserve to escrow (no real "hold" balance):
  /// Maps a logical hold to a transfer from user -> platform (escrow) and journals both sides.
  /// Requires `form_out.kind = TxKind::Transfer` and uses the same `idempotency_key` for both entries.
  pub async fn hold(
    pool: &mut DbPool<'_>,
    form_out: &WalletTransactionInsertForm,
  ) -> FastJobResult<Wallet> {
    let amount = form_out.amount;
    Self::validate_positive_amount(amount)?;
    if !matches!(form_out.kind, TxKind::Transfer) {
      return Err(FastJobErrorType::InvalidField("hold requires kind=Transfer".into()).into());
    }

    // Figure out the platform (escrow) wallet id first
    let platform_id = {
      let conn = &mut get_conn(pool).await?;
      Self::platform_wallet_id(conn).await?
    };

    // Build the mirrored incoming form for the platform wallet
    let mut form_in = form_out.clone();
    form_in.wallet_id = platform_id;
    form_in.description = if form_in.description.is_empty() {
      "escrow reserve".to_string()
    } else {
      format!("{} (escrow)", form_in.description)
    };

    // Execute paired transfer + journal atomically
    Self::transfer_between_wallets(pool, form_out, &form_in).await?;

    // Return the updated user wallet
    let conn = &mut get_conn(pool).await?;
    let updated = Self::load_for_update(conn, form_out.wallet_id).await?;
    Ok(updated)
  }

  /// Reserve funds inside the same wallet (three-balance model): a -= amt; o += amt
  /// Journal with the provided form (e.g., kind=Transfer + your description/idempotency).
  pub async fn reserve(
    pool: &mut DbPool<'_>,
    form: &WalletTransactionInsertForm,
  ) -> FastJobResult<Wallet> {
    let amount = form.amount;
    Self::validate_positive_amount(amount)?;

    let conn = &mut get_conn(pool).await?;
    let w = conn
      .run_transaction(|conn| {
        async move {
          // apply state change
          let _ = Self::apply_op_on(conn, form.wallet_id, BalanceOp::Reserve, amount).await?;
          // journal single-side entry
          let _ = Self::insert_wallet_tx(conn, form).await?;
          // return updated wallet
          let w = Self::load_for_update(conn, form.wallet_id).await?;
          Ok::<_, app_108jobs_utils::error::FastJobError>(w)
        }
        .scope_boxed()
      })
      .await?;
    Ok(w)
  }

  /// Release previously reserved funds: o -= amt; a += amt
  pub async fn release(
    pool: &mut DbPool<'_>,
    form: &WalletTransactionInsertForm,
  ) -> FastJobResult<Wallet> {
    let amount = form.amount;
    Self::validate_positive_amount(amount)?;

    let conn = &mut get_conn(pool).await?;
    let w = conn
      .run_transaction(|conn| {
        async move {
          let _ = Self::apply_op_on(conn, form.wallet_id, BalanceOp::Release, amount).await?;
          let _ = Self::insert_wallet_tx(conn, form).await?;
          let w = Self::load_for_update(conn, form.wallet_id).await?;
          Ok::<_, app_108jobs_utils::error::FastJobError>(w)
        }
        .scope_boxed()
      })
      .await?;
    Ok(w)
  }

  /// Capture part/all of the reserved funds: o -= amt; t -= amt (final debit)
  pub async fn capture(
    pool: &mut DbPool<'_>,
    form: &WalletTransactionInsertForm,
  ) -> FastJobResult<Wallet> {
    let amount = form.amount;
    Self::validate_positive_amount(amount)?;

    let conn = &mut get_conn(pool).await?;
    let w = conn
      .run_transaction(|conn| {
        async move {
          let _ = Self::apply_op_on(conn, form.wallet_id, BalanceOp::Capture, amount).await?;
          let _ = Self::insert_wallet_tx(conn, form).await?;
          let w = Self::load_for_update(conn, form.wallet_id).await?;
          Ok::<_, app_108jobs_utils::error::FastJobError>(w)
        }
        .scope_boxed()
      })
      .await?;
    Ok(w)
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


  /// Validate paired transfer forms and return (from, to, amount_rounded).
  fn validate_transfer_pair(
    form_out: &WalletTransactionInsertForm,
    form_in:  &WalletTransactionInsertForm,
  ) -> FastJobResult<(WalletId, WalletId, Coin)> {
    // both must be Transfer
    if !matches!(form_out.kind, TxKind::Transfer) || !matches!(form_in.kind, TxKind::Transfer) {
      return Err(FastJobErrorType::InvalidField("Both forms must be kind=Transfer".into()).into());
    }
    // idempotency must match
    if form_out.idempotency_key != form_in.idempotency_key {
      return Err(FastJobErrorType::InvalidField("Idempotency key must match for transfer pair".into()).into());
    }
    // wallet ids
    let from = form_out.wallet_id;
    let to   = form_in.wallet_id;
    if from == to {
      return Err(FastJobErrorType::InvalidField("Cannot transfer to the same wallet".into()).into());
    }
    // amounts must match exactly and be positive
    let a_out = form_out.amount;
    let a_in  = form_in.amount;
    if a_out <= 0 { return Err(FastJobErrorType::InvalidField("Amount must be positive".into()).into()); }
    if a_out != a_in {
      return Err(FastJobErrorType::InvalidField("Mismatched amounts for transfer pair".into()).into());
    }
    Ok((from, to, a_out))
  }

  /// Paired transfer: requires two mirrored forms (outgoing + incoming) with the same idempotency_key.
  pub async fn transfer_between_wallets(
    pool: &mut DbPool<'_>,
    form_out: &WalletTransactionInsertForm, // from -> ...
    form_in:  &WalletTransactionInsertForm, // ... -> to
  ) -> FastJobResult<()> {
    let (from, to, amount) = Self::validate_transfer_pair(form_out, form_in)?;

    let conn = &mut get_conn(pool).await?;
    conn.run_transaction(|conn| {
      async move {
        // move balances first
        Self::move_funds(conn, from, to, amount).await?;
        // journal both sides
        let _ = Self::insert_wallet_tx(conn, form_out).await?;
        let _ = Self::insert_wallet_tx(conn, form_in).await?;
        Ok::<_, app_108jobs_utils::error::FastJobError>(())
      }.scope_boxed()
    }).await?;
    Ok(())
  }

  /// Deposit from platform -> user: must create a pair of journal rows
  pub async fn deposit_from_platform(
    pool: &mut DbPool<'_>,
    form: &WalletTransactionInsertForm,
    coin_id: CoinId,
    platform_wallet_id: WalletId,
  ) -> FastJobResult<Wallet> {
    let amount = form.amount;
    Self::validate_positive_amount(amount)?;

    let conn = &mut get_conn(pool).await?;
    conn
      .run_transaction(|conn| {
        async move {
          // move funds: platform -> user
          Self::move_funds(conn, platform_wallet_id, form.wallet_id, amount).await?;
          let _ = CoinModel::update_balance(conn, coin_id, -amount).await?;
          // journal user side
          let _ = Self::insert_wallet_tx(conn, form).await?;
          // mirrored platform-side entry
          let mut mirror = form.clone();
          mirror.wallet_id = platform_wallet_id;
          mirror.description = format!("platform counter: {}", mirror.description);
          let _ = Self::insert_wallet_tx(conn, &mirror).await?;
          // return updated user wallet
          let w = Self::load_for_update(conn, form.wallet_id).await?;
          Ok::<_, app_108jobs_utils::error::FastJobError>(w)
        }
        .scope_boxed()
      })
      .await
  }

  /// Withdraw to platform (user -> platform): must create a pair of journal rows
  pub async fn withdraw_to_platform(
    pool: &mut DbPool<'_>,
    form: &WalletTransactionInsertForm,
    coin_id: CoinId,
    platform_wallet_id: WalletId,
  ) -> FastJobResult<Wallet> {
    let amount = form.amount;
    Self::validate_positive_amount(amount)?;

    let conn = &mut get_conn(pool).await?;
    return conn
      .run_transaction(|conn| {
        async move {
          // move funds: user -> platform
          Self::move_funds(conn, form.wallet_id, platform_wallet_id, amount).await?;
          let _ = CoinModel::update_balance(conn.into(), coin_id, amount).await?;
          // journal user side
          let _ = Self::insert_wallet_tx(conn, form).await?;
          // mirrored platform-side entry
          let mut mirror = form.clone();
          mirror.wallet_id = platform_wallet_id;
          mirror.description = format!("platform counter: {}", mirror.description);
          let _ = Self::insert_wallet_tx(conn, &mirror).await?;
          // return updated user wallet
          let w = Self::load_for_update(conn, form.wallet_id).await?;
          Ok::<_, app_108jobs_utils::error::FastJobError>(w)
        }
        .scope_boxed()
      })
      .await;
  }
  async fn move_funds(
    conn: &mut diesel_async::AsyncPgConnection,
    from_wallet: WalletId,
    to_wallet: WalletId,
    amount: Coin,
  ) -> FastJobResult<()> {
    // basic validation
    Self::validate_positive_amount(amount)?;
    if from_wallet == to_wallet {
      return Err(FastJobErrorType::InvalidField("Cannot transfer to the same wallet".into()).into());
    }

    // Direct transfer: decrease from.available & from.total; increase to.available & to.total
    let _ = Self::apply_op_on(conn, from_wallet, BalanceOp::TransferOut, amount).await?;
    let _ = Self::apply_op_on(conn, to_wallet,   BalanceOp::TransferIn,  amount).await?;
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
