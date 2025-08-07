use bigdecimal::BigDecimal;
use diesel::prelude::*;
use lemmy_db_schema::newtypes::{WalletId, LocalUserId};
use lemmy_utils::error::{FastJobError, FastJobErrorType, FastJobResult};

/// Optimistic locking implementation for wallet operations
pub struct WalletOptimisticLocking;

impl WalletOptimisticLocking {
    /// Update wallet balance with optimistic locking
    pub async fn update_balance_safely(
        pool: &mut DbPool<'_>,
        wallet_id: WalletId,
        amount_change: BigDecimal,
        operation_type: &str,
    ) -> FastJobResult<Wallet> {
        let conn = &mut get_conn(pool).await?;
        
        // Step 1: Read current wallet state (including version)
        let current_wallet = wallet::table
            .find(wallet_id)
            .first::<Wallet>(conn)
            .await
            .map_err(|_| FastJobErrorType::WalletNotFound)?;
        
        // Step 2: Calculate new balance
        let new_balance = &current_wallet.available_balance + &amount_change;
        
        // Step 3: Validate business rules
        if new_balance < BigDecimal::from(0) {
            return Err(FastJobErrorType::InsufficientBalance)?;
        }
        
        // Step 4: Update with version check (THE KEY PART!)
        let updated_rows = diesel::update(wallet::table)
            .filter(wallet::id.eq(wallet_id))
            .filter(wallet::version.eq(current_wallet.version)) // ✅ Version check!
            .set((
                wallet::available_balance.eq(&new_balance),
                wallet::updated_at.eq(chrono::Utc::now()),
                // Note: version is auto-incremented by trigger
            ))
            .execute(conn)
            .await?;
        
        // Step 5: Check if update succeeded
        if updated_rows == 0 {
            // Version mismatch = concurrent modification detected!
            return Err(FastJobErrorType::ConcurrentModification(
                format!("Wallet {} was modified by another transaction", wallet_id.0)
            ))?;
        }
        
        // Step 6: Return updated wallet
        let updated_wallet = wallet::table
            .find(wallet_id)
            .first::<Wallet>(conn)
            .await?;
            
        println!("✅ {} successful: ${} (version {} -> {})", 
                operation_type, amount_change, 
                current_wallet.version, updated_wallet.version);
                
        Ok(updated_wallet)
    }
    
    /// Optimistic locking with retry logic
    pub async fn update_balance_with_retry(
        pool: &mut DbPool<'_>,
        wallet_id: WalletId,
        amount_change: BigDecimal,
        operation_type: &str,
        max_retries: u32,
    ) -> FastJobResult<Wallet> {
        let mut attempts = 0;
        
        loop {
            attempts += 1;
            
            match Self::update_balance_safely(pool, wallet_id, amount_change.clone(), operation_type).await {
                Ok(wallet) => {
                    if attempts > 1 {
                        println!("✅ Success after {} attempts", attempts);
                    }
                    return Ok(wallet);
                },
                Err(FastJobError { 
                    error_type: FastJobErrorType::ConcurrentModification(_), 
                    .. 
                }) if attempts < max_retries => {
                    // Concurrent modification detected, retry with exponential backoff
                    let delay_ms = 50 * (2_u64.pow(attempts - 1)); // 50ms, 100ms, 200ms, 400ms...
                    println!("⚠️  Attempt {} failed (concurrent modification), retrying in {}ms...", 
                            attempts, delay_ms);
                    
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    continue;
                },
                Err(e) => {
                    // Other errors or max retries exceeded
                    return Err(e);
                }
            }
        }
    }
    
    /// Complex transaction with multiple wallet updates
    pub async fn transfer_money_safely(
        pool: &mut DbPool<'_>,
        from_wallet_id: WalletId,
        to_wallet_id: WalletId,
        amount: BigDecimal,
    ) -> FastJobResult<(Wallet, Wallet)> {
        let conn = &mut get_conn(pool).await?;
        
        // Start database transaction for atomicity
        conn.begin_test_transaction().await?;
        
        // Read both wallets with their versions
        let from_wallet = wallet::table
            .find(from_wallet_id)
            .first::<Wallet>(conn)
            .await?;
            
        let to_wallet = wallet::table
            .find(to_wallet_id)
            .first::<Wallet>(conn)
            .await?;
        
        // Validate sender has sufficient balance
        if from_wallet.available_balance < amount {
            conn.rollback_test_transaction().await?;
            return Err(FastJobErrorType::InsufficientBalance)?;
        }
        
        // Update sender wallet (optimistic lock check)
        let sender_updated_rows = diesel::update(wallet::table)
            .filter(wallet::id.eq(from_wallet_id))
            .filter(wallet::version.eq(from_wallet.version)) // ✅ Version check!
            .set((
                wallet::available_balance.eq(&from_wallet.available_balance - &amount),
                wallet::updated_at.eq(chrono::Utc::now()),
            ))
            .execute(conn)
            .await?;
            
        if sender_updated_rows == 0 {
            conn.rollback_test_transaction().await?;
            return Err(FastJobErrorType::ConcurrentModification(
                format!("Sender wallet {} was modified", from_wallet_id.0)
            ))?;
        }
        
        // Update receiver wallet (optimistic lock check)  
        let receiver_updated_rows = diesel::update(wallet::table)
            .filter(wallet::id.eq(to_wallet_id))
            .filter(wallet::version.eq(to_wallet.version)) // ✅ Version check!
            .set((
                wallet::available_balance.eq(&to_wallet.available_balance + &amount),
                wallet::updated_at.eq(chrono::Utc::now()),
            ))
            .execute(conn)
            .await?;
            
        if receiver_updated_rows == 0 {
            conn.rollback_test_transaction().await?;
            return Err(FastJobErrorType::ConcurrentModification(
                format!("Receiver wallet {} was modified", to_wallet_id.0)
            ))?;
        }
        
        // Get updated wallets
        let updated_from = wallet::table.find(from_wallet_id).first::<Wallet>(conn).await?;
        let updated_to = wallet::table.find(to_wallet_id).first::<Wallet>(conn).await?;
        
        conn.commit_test_transaction().await?;
        
        println!("✅ Transfer complete: ${} from wallet {} (v{}) to wallet {} (v{})", 
                amount, from_wallet_id.0, updated_from.version, 
                to_wallet_id.0, updated_to.version);
                
        Ok((updated_from, updated_to))
    }
    
    /// Batch update with optimistic locking
    pub async fn batch_update_wallets(
        pool: &mut DbPool<'_>,
        updates: Vec<(WalletId, BigDecimal)>, // (wallet_id, amount_change)
    ) -> FastJobResult<Vec<Wallet>> {
        let conn = &mut get_conn(pool).await?;
        conn.begin_test_transaction().await?;
        
        let mut updated_wallets = Vec::new();
        
        // Read all wallets first (with versions)
        let mut wallet_versions = std::collections::HashMap::new();
        for (wallet_id, _) in &updates {
            let wallet = wallet::table
                .find(*wallet_id)
                .first::<Wallet>(conn)
                .await?;
            wallet_versions.insert(*wallet_id, wallet);
        }
        
        // Apply all updates with version checks
        for (wallet_id, amount_change) in &updates {
            let current_wallet = wallet_versions.get(wallet_id).unwrap();
            let new_balance = &current_wallet.available_balance + amount_change;
            
            if new_balance < BigDecimal::from(0) {
                conn.rollback_test_transaction().await?;
                return Err(FastJobErrorType::InsufficientBalance)?;
            }
            
            let updated_rows = diesel::update(wallet::table)
                .filter(wallet::id.eq(*wallet_id))
                .filter(wallet::version.eq(current_wallet.version)) // ✅ Version check!
                .set((
                    wallet::available_balance.eq(&new_balance),
                    wallet::updated_at.eq(chrono::Utc::now()),
                ))
                .execute(conn)
                .await?;
                
            if updated_rows == 0 {
                conn.rollback_test_transaction().await?;
                return Err(FastJobErrorType::ConcurrentModification(
                    format!("Wallet {} was modified during batch update", wallet_id.0)
                ))?;
            }
        }
        
        // Collect updated wallets
        for (wallet_id, _) in &updates {
            let updated_wallet = wallet::table
                .find(*wallet_id)
                .first::<Wallet>(conn)
                .await?;
            updated_wallets.push(updated_wallet);
        }
        
        conn.commit_test_transaction().await?;
        println!("✅ Batch update completed for {} wallets", updates.len());
        
        Ok(updated_wallets)
    }
}

/// Custom error types for optimistic locking
#[derive(Debug)]
pub enum WalletLockError {
    ConcurrentModification(String),
    InsufficientBalance,
    WalletNotFound,
    DatabaseError(diesel::result::Error),
}

/// Utility for handling concurrent operations
pub struct ConcurrentWalletManager;

impl ConcurrentWalletManager {
    /// Safe wrapper that handles all optimistic locking concerns
    pub async fn execute_wallet_operation<F, T>(
        pool: &mut DbPool<'_>,
        operation: F,
        max_retries: u32,
    ) -> FastJobResult<T>
    where
        F: Fn(&mut AsyncPgConnection) -> BoxFuture<'_, FastJobResult<T>>,
    {
        let mut attempts = 0;
        
        loop {
            attempts += 1;
            let conn = &mut get_conn(pool).await?;
            
            match operation(conn).await {
                Ok(result) => return Ok(result),
                Err(FastJobError { 
                    error_type: FastJobErrorType::ConcurrentModification(_), 
                    .. 
                }) if attempts < max_retries => {
                    // Exponential backoff with jitter
                    let base_delay = 100 * (2_u64.pow(attempts - 1));
                    let jitter = fastrand::u64(0..=base_delay / 4); // Add 0-25% jitter
                    let delay_ms = base_delay + jitter;
                    
                    println!("🔄 Retry {} after {}ms (concurrent modification)", attempts, delay_ms);
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    continue;
                },
                Err(e) => return Err(e),
            }
        }
    }
}