/// Practical usage examples of optimistic locking in wallet operations

#[tokio::test]
async fn test_concurrent_wallet_updates() {
    let pool = setup_test_db().await;
    let wallet_id = create_test_wallet(&pool, BigDecimal::from(1000)).await;
    
    println!("🧪 Testing concurrent wallet updates...");
    
    // Simulate 10 concurrent $10 withdrawals
    let mut handles = vec![];
    
    for i in 0..10 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let result = WalletOptimisticLocking::update_balance_with_retry(
                &mut pool_clone,
                wallet_id,
                BigDecimal::from(-10), // $10 withdrawal
                &format!("Concurrent withdrawal {}", i),
                5, // max 5 retries
            ).await;
            
            match result {
                Ok(wallet) => println!("✅ Withdrawal {} succeeded, balance: ${}", i, wallet.available_balance),
                Err(e) => println!("❌ Withdrawal {} failed: {:?}", i, e),
            }
            
            result
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    let results: Vec<_> = futures::future::join_all(handles).await;
    let successful_operations = results.iter().filter(|r| r.as_ref().unwrap().is_ok()).count();
    
    println!("📊 Results: {}/{} operations succeeded", successful_operations, results.len());
    
    // Verify final balance
    let final_wallet = get_wallet(&pool, wallet_id).await.unwrap();
    let expected_balance = BigDecimal::from(1000 - (successful_operations as i32 * 10));
    
    assert_eq!(final_wallet.available_balance, expected_balance);
    println!("✅ Final balance correct: ${}", final_wallet.available_balance);
}

/// Example: Job payment with optimistic locking
pub async fn process_job_payment_safely(
    pool: &mut DbPool<'_>,
    employer_id: LocalUserId,
    freelancer_id: LocalUserId,
    amount: BigDecimal,
    billing_id: BillingId,
) -> FastJobResult<TransactionId> {
    println!("💰 Processing job payment: ${} from employer {} to freelancer {}", 
             amount, employer_id.0, freelancer_id.0);
             
    // Use optimistic locking wrapper
    ConcurrentWalletManager::execute_wallet_operation(
        pool,
        |conn| {
            Box::pin(async move {
                // Get wallet IDs
                let employer_wallet_id = get_wallet_id_for_user(conn, employer_id).await?;
                let freelancer_wallet_id = get_wallet_id_for_user(conn, freelancer_id).await?;
                
                // Read current wallet states (with versions)
                let employer_wallet = wallet::table
                    .find(employer_wallet_id)
                    .first::<Wallet>(conn)
                    .await?;
                    
                let freelancer_wallet = wallet::table
                    .find(freelancer_wallet_id) 
                    .first::<Wallet>(conn)
                    .await?;
                
                // Validate employer has sufficient escrow balance
                if employer_wallet.escrow_balance < amount {
                    return Err(FastJobErrorType::InsufficientBalance)?;
                }
                
                // Create transaction record first
                let transaction = create_transaction_record(
                    conn,
                    Some(employer_id),
                    Some(freelancer_id),
                    amount.clone(),
                    TransactionType::JobPayment,
                    format!("Job payment for billing {}", billing_id.0),
                    Some(billing_id),
                ).await?;
                
                // Update employer wallet (remove from escrow)
                let employer_updated = diesel::update(wallet::table)
                    .filter(wallet::id.eq(employer_wallet_id))
                    .filter(wallet::version.eq(employer_wallet.version)) // 🔒 Optimistic lock!
                    .set((
                        wallet::escrow_balance.eq(&employer_wallet.escrow_balance - &amount),
                        wallet::last_transaction_at.eq(chrono::Utc::now()),
                    ))
                    .execute(conn)
                    .await?;
                    
                if employer_updated == 0 {
                    return Err(FastJobErrorType::ConcurrentModification(
                        format!("Employer wallet {} was modified", employer_wallet_id.0)
                    ))?;
                }
                
                // Update freelancer wallet (add to available balance)
                let freelancer_updated = diesel::update(wallet::table)
                    .filter(wallet::id.eq(freelancer_wallet_id))
                    .filter(wallet::version.eq(freelancer_wallet.version)) // 🔒 Optimistic lock!
                    .set((
                        wallet::available_balance.eq(&freelancer_wallet.available_balance + &amount),
                        wallet::last_transaction_at.eq(chrono::Utc::now()),
                    ))
                    .execute(conn)
                    .await?;
                    
                if freelancer_updated == 0 {
                    return Err(FastJobErrorType::ConcurrentModification(
                        format!("Freelancer wallet {} was modified", freelancer_wallet_id.0)
                    ))?;
                }
                
                // Mark transaction as completed
                diesel::update(transaction_table::table)
                    .filter(transaction_table::id.eq(transaction.id))
                    .set((
                        transaction_table::status.eq(TransactionStatus::Completed),
                        transaction_table::completed_at.eq(Some(chrono::Utc::now())),
                    ))
                    .execute(conn)
                    .await?;
                
                println!("✅ Job payment completed successfully!");
                Ok(transaction.id)
            })
        },
        3, // max 3 retries
    ).await
}

/// Example: Batch salary payments with optimistic locking
pub async fn process_monthly_salaries(
    pool: &mut DbPool<'_>,
    company_wallet_id: WalletId,
    salary_payments: Vec<(LocalUserId, BigDecimal)>, // (employee_id, salary_amount)
) -> FastJobResult<Vec<TransactionId>> {
    println!("💼 Processing {} monthly salary payments...", salary_payments.len());
    
    let total_amount: BigDecimal = salary_payments.iter().map(|(_, amount)| amount).sum();
    println!("💰 Total payout: ${}", total_amount);
    
    ConcurrentWalletManager::execute_wallet_operation(
        pool,
        |conn| {
            Box::pin(async move {
                conn.begin_test_transaction().await?;
                
                // Check company has sufficient funds
                let company_wallet = wallet::table
                    .find(company_wallet_id)
                    .first::<Wallet>(conn)
                    .await?;
                    
                if company_wallet.available_balance < total_amount {
                    return Err(FastJobErrorType::InsufficientBalance)?;
                }
                
                // Deduct total from company wallet
                let company_updated = diesel::update(wallet::table)
                    .filter(wallet::id.eq(company_wallet_id))
                    .filter(wallet::version.eq(company_wallet.version)) // 🔒 Optimistic lock!
                    .set(wallet::available_balance.eq(&company_wallet.available_balance - &total_amount))
                    .execute(conn)
                    .await?;
                    
                if company_updated == 0 {
                    conn.rollback_test_transaction().await?;
                    return Err(FastJobErrorType::ConcurrentModification(
                        "Company wallet was modified during salary processing".to_string()
                    ))?;
                }
                
                let mut transaction_ids = Vec::new();
                
                // Process each salary payment
                for (employee_id, salary_amount) in salary_payments {
                    let employee_wallet_id = get_wallet_id_for_user(conn, employee_id).await?;
                    
                    let employee_wallet = wallet::table
                        .find(employee_wallet_id)
                        .first::<Wallet>(conn)
                        .await?;
                    
                    // Update employee wallet
                    let employee_updated = diesel::update(wallet::table)
                        .filter(wallet::id.eq(employee_wallet_id))
                        .filter(wallet::version.eq(employee_wallet.version)) // 🔒 Optimistic lock!
                        .set((
                            wallet::available_balance.eq(&employee_wallet.available_balance + &salary_amount),
                            wallet::last_transaction_at.eq(chrono::Utc::now()),
                        ))
                        .execute(conn)
                        .await?;
                        
                    if employee_updated == 0 {
                        conn.rollback_test_transaction().await?;
                        return Err(FastJobErrorType::ConcurrentModification(
                            format!("Employee {} wallet was modified", employee_id.0)
                        ))?;
                    }
                    
                    // Create transaction record
                    let transaction = create_transaction_record(
                        conn,
                        None, // Company system payment
                        Some(employee_id),
                        salary_amount,
                        TransactionType::Bonus, // Using Bonus for salary
                        format!("Monthly salary payment"),
                        None,
                    ).await?;
                    
                    transaction_ids.push(transaction.id);
                }
                
                conn.commit_test_transaction().await?;
                println!("✅ All {} salary payments processed successfully!", transaction_ids.len());
                
                Ok(transaction_ids)
            })
        },
        5, // max 5 retries for batch operations
    ).await
}

/// High-frequency trading scenario
pub async fn stress_test_optimistic_locking() -> FastJobResult<()> {
    let pool = setup_test_db().await;
    let wallet_id = create_test_wallet(&pool, BigDecimal::from(10000)).await;
    
    println!("🚀 Stress testing optimistic locking with 100 concurrent operations...");
    
    let start_time = std::time::Instant::now();
    let mut handles = vec![];
    
    // Launch 100 concurrent operations
    for i in 0..100 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let amount = if i % 2 == 0 { 
                BigDecimal::from(10)  // Deposit
            } else { 
                BigDecimal::from(-5)  // Withdrawal
            };
            
            WalletOptimisticLocking::update_balance_with_retry(
                &mut pool_clone,
                wallet_id,
                amount,
                &format!("Stress test operation {}", i),
                10, // High retry count for stress test
            ).await
        });
        handles.push(handle);
    }
    
    // Wait for completion
    let results = futures::future::join_all(handles).await;
    let successful = results.iter().filter(|r| r.as_ref().unwrap().is_ok()).count();
    let duration = start_time.elapsed();
    
    println!("📊 Stress test results:");
    println!("  • Operations: {}/{} successful", successful, results.len());
    println!("  • Duration: {:?}", duration);
    println!("  • Rate: {:.2} ops/sec", results.len() as f64 / duration.as_secs_f64());
    
    // Verify final balance integrity
    let final_wallet = get_wallet(&pool, wallet_id).await.unwrap();
    let expected_change = successful as i32 * 5; // Net +$5 per successful operation
    let expected_balance = BigDecimal::from(10000 + expected_change);
    
    assert_eq!(final_wallet.available_balance, expected_balance);
    println!("✅ Balance integrity maintained: ${}", final_wallet.available_balance);
    
    Ok(())
}