use bigdecimal::BigDecimal;
use chrono::Utc;
use lemmy_db_schema::newtypes::{LocalUserId, TransactionId};
use lemmy_db_schema_file::enums::{TransactionType, TransactionStatus};

/// Integrated wallet and transaction processing
pub struct WalletTransactionProcessor;

impl WalletTransactionProcessor {
    /// Process a transaction with atomic wallet updates
    pub async fn process_transaction(
        pool: &mut DbPool<'_>,
        from_user_id: Option<LocalUserId>,
        to_user_id: Option<LocalUserId>,
        amount: BigDecimal,
        transaction_type: TransactionType,
        description: String,
        billing_id: Option<BillingId>,
    ) -> FastJobResult<TransactionId> {
        let conn = &mut get_conn(pool).await?;
        
        // Start database transaction for atomicity
        conn.begin_test_transaction().await?;
        
        // Step 1: Create transaction record (Pending status)
        let transaction = Self::create_transaction_record(
            conn,
            from_user_id,
            to_user_id,
            amount.clone(),
            transaction_type,
            description,
            billing_id,
        ).await?;
        
        // Step 2: Update wallet balances based on transaction type
        match transaction_type {
            TransactionType::JobPayment => {
                Self::process_job_payment(conn, from_user_id, to_user_id, &amount).await?;
            },
            TransactionType::EscrowDeposit => {
                Self::process_escrow_deposit(conn, from_user_id.unwrap(), &amount).await?;
            },
            TransactionType::EscrowRelease => {
                Self::process_escrow_release(conn, from_user_id, to_user_id.unwrap(), &amount).await?;
            },
            TransactionType::AdminTopUp => {
                Self::process_admin_top_up(conn, to_user_id.unwrap(), &amount).await?;
            },
            TransactionType::UserWithdraw => {
                Self::process_user_withdrawal(conn, from_user_id.unwrap(), &amount).await?;
            },
            // ... other transaction types
            _ => return Err(FastJobErrorType::InvalidField("Unsupported transaction type".to_string()))?,
        }
        
        // Step 3: Mark transaction as completed
        Self::complete_transaction(conn, transaction.id).await?;
        
        // Step 4: Update wallet version for optimistic locking
        Self::increment_wallet_versions(conn, from_user_id, to_user_id).await?;
        
        conn.commit_test_transaction().await?;
        Ok(transaction.id)
    }
    
    /// Job payment: Employer -> Freelancer (from escrow)
    async fn process_job_payment(
        conn: &mut AsyncPgConnection,
        employer_id: Option<LocalUserId>,
        freelancer_id: Option<LocalUserId>, 
        amount: &BigDecimal,
    ) -> FastJobResult<()> {
        let employer_id = employer_id.ok_or(FastJobErrorType::InvalidField("Missing employer ID".to_string()))?;
        let freelancer_id = freelancer_id.ok_or(FastJobErrorType::InvalidField("Missing freelancer ID".to_string()))?;
        
        // Move money from employer's escrow to freelancer's available balance
        diesel::update(wallet::table)
            .filter(wallet::id.eq(Self::get_wallet_id_for_user(conn, employer_id).await?))
            .set(wallet::escrow_balance.eq(wallet::escrow_balance - amount))
            .execute(conn)
            .await?;
            
        diesel::update(wallet::table)
            .filter(wallet::id.eq(Self::get_wallet_id_for_user(conn, freelancer_id).await?))
            .set((
                wallet::available_balance.eq(wallet::available_balance + amount),
                wallet::last_transaction_at.eq(Utc::now()),
            ))
            .execute(conn)
            .await?;
            
        Ok(())
    }
    
    /// Escrow deposit: Available balance -> Escrow balance
    async fn process_escrow_deposit(
        conn: &mut AsyncPgConnection,
        user_id: LocalUserId,
        amount: &BigDecimal,
    ) -> FastJobResult<()> {
        let wallet_id = Self::get_wallet_id_for_user(conn, user_id).await?;
        
        // Check sufficient balance
        let current_wallet = wallet::table
            .find(wallet_id)
            .first::<Wallet>(conn)
            .await?;
            
        if current_wallet.available_balance < *amount {
            return Err(FastJobErrorType::InsufficientBalance)?;
        }
        
        // Move from available to escrow
        diesel::update(wallet::table.find(wallet_id))
            .set((
                wallet::available_balance.eq(wallet::available_balance - amount),
                wallet::escrow_balance.eq(wallet::escrow_balance + amount),
                wallet::last_transaction_at.eq(Utc::now()),
            ))
            .execute(conn)
            .await?;
            
        Ok(())
    }
    
    /// Track pending transactions in wallet
    pub async fn create_pending_transaction(
        pool: &mut DbPool<'_>,
        user_id: LocalUserId,
        amount: BigDecimal,
        is_incoming: bool,
    ) -> FastJobResult<()> {
        let conn = &mut get_conn(pool).await?;
        let wallet_id = Self::get_wallet_id_for_user(conn, user_id).await?;
        
        if is_incoming {
            // Increase pending_in
            diesel::update(wallet::table.find(wallet_id))
                .set(wallet::pending_in.eq(wallet::pending_in + &amount))
                .execute(conn)
                .await?;
        } else {
            // Increase pending_out  
            diesel::update(wallet::table.find(wallet_id))
                .set(wallet::pending_out.eq(wallet::pending_out + &amount))
                .execute(conn)
                .await?;
        }
        
        Ok(())
    }
    
    /// Clear pending amounts when transaction completes
    pub async fn clear_pending_transaction(
        pool: &mut DbPool<'_>,
        user_id: LocalUserId,
        amount: BigDecimal,
        is_incoming: bool,
    ) -> FastJobResult<()> {
        let conn = &mut get_conn(pool).await?;
        let wallet_id = Self::get_wallet_id_for_user(conn, user_id).await?;
        
        if is_incoming {
            diesel::update(wallet::table.find(wallet_id))
                .set(wallet::pending_in.eq(wallet::pending_in - &amount))
                .execute(conn)
                .await?;
        } else {
            diesel::update(wallet::table.find(wallet_id))
                .set(wallet::pending_out.eq(wallet::pending_out - &amount))
                .execute(conn)
                .await?;
        }
        
        Ok(())
    }
    
    /// Get user's complete financial picture
    pub async fn get_wallet_with_transaction_summary(
        pool: &mut DbPool<'_>,
        user_id: LocalUserId,
    ) -> FastJobResult<WalletTransactionSummary> {
        let conn = &mut get_conn(pool).await?;
        
        // Get wallet data
        let wallet = Self::get_wallet_for_user(conn, user_id).await?;
        
        // Get recent transaction count
        let recent_transaction_count = transaction::table
            .filter(
                transaction::from_user_id.eq(user_id)
                    .or(transaction::to_user_id.eq(user_id))
            )
            .filter(transaction::created_at.gt(Utc::now() - chrono::Duration::days(30)))
            .count()
            .get_result::<i64>(conn)
            .await?;
        
        // Calculate total balance (available + escrow - pending_out)
        let total_balance = &wallet.available_balance + &wallet.escrow_balance - &wallet.pending_out;
        
        Ok(WalletTransactionSummary {
            wallet,
            total_balance,
            recent_transaction_count,
            last_activity: wallet.last_transaction_at,
        })
    }
}

/// Combined wallet and transaction data
#[derive(Debug, Serialize)]
pub struct WalletTransactionSummary {
    pub wallet: Wallet,
    pub total_balance: BigDecimal,
    pub recent_transaction_count: i64,
    pub last_activity: Option<DateTime<Utc>>,
}