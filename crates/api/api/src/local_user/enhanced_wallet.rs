use lemmy_db_schema::source::transaction::TransactionInsertForm;
use chrono::Utc;
use actix_web::web::{Data, Json, Query};
use bigdecimal::BigDecimal;
use lemmy_api_common::wallet::{
    GetWalletResponse, WithdrawWallet, GetTransactions, GetTransactionsResponse, TransactionResponse,
  EnhancedWalletOperationResponse
};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::{LocalUserId, PersonId};
use lemmy_db_schema_file::enums::{TransactionType, TransactionStatus};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_wallet::WalletView;
use lemmy_db_views_transaction::TransactionView;
use lemmy_utils::error::{FastJobResult, FastJobErrorType};

/// Enhanced wallet management with optimistic locking
pub struct EnhancedWalletManager;

impl EnhancedWalletManager {
    /// Get comprehensive wallet information with transaction summary
    pub async fn get_wallet_enhanced(
        context: Data<FastJobContext>,
        local_user_view: LocalUserView,
    ) -> FastJobResult<Json<GetWalletResponse>> {
        let user_id = local_user_view.local_user.id;

        // Get wallet information
        let wallet_opt = WalletView::read_by_user(&mut context.pool(), user_id).await?;
        
        match wallet_opt {
            Some(wallet_view) => {
                let wallet = &wallet_view.wallet;
                let total_balance = &wallet.available_balance 
                    + &wallet.escrow_balance 
                    + &wallet.pending_in 
                    - &wallet.pending_out;

                Ok(Json(GetWalletResponse {
                    wallet_id: Some(wallet.id),
                    available_balance: wallet.available_balance.clone(),
                    escrow_balance: wallet.escrow_balance.clone(),
                    pending_in: wallet.pending_in.clone(),
                    pending_out: wallet.pending_out.clone(),
                    reserved_balance: wallet.reserved_balance.clone(),
                    total_balance,
                    is_frozen: wallet.is_frozen,
                    freeze_reason: wallet.freeze_reason.clone(),
                    currency: wallet.currency.clone(),
                    version: wallet.version,
                    last_transaction_at: wallet.last_transaction_at.map(|dt| dt.to_rfc3339()),
                    recent_transaction_count: TransactionView::get_recent_transaction_count(&mut context.pool(), local_user_view.local_user.person_id, 30).await.unwrap_or(0),
                }))
            },
            None => {
                // No wallet found, return empty wallet response
                Ok(Json(GetWalletResponse {
                    wallet_id: None,
                    available_balance: BigDecimal::from(0),
                    escrow_balance: BigDecimal::from(0),
                    pending_in: BigDecimal::from(0),
                    pending_out: BigDecimal::from(0),
                    reserved_balance: BigDecimal::from(0),
                    total_balance: BigDecimal::from(0),
                    is_frozen: false,
                    freeze_reason: None,
                    currency: "USD".to_string(),
                    version: 1,
                    last_transaction_at: None,
                    recent_transaction_count: 0,
                }))
            }
        }
    }


    /// Withdraw money with optimistic locking
    pub async fn withdraw_money(
        data: Json<WithdrawWallet>,
        context: Data<FastJobContext>,
        local_user_view: LocalUserView,
    ) -> FastJobResult<Json<EnhancedWalletOperationResponse>> {
        let user_id = local_user_view.local_user.id;
        let person_id = local_user_view.local_user.person_id;

        // Validate amount
        if data.amount <= BigDecimal::from(0) {
            return Err(FastJobErrorType::InvalidField("Amount must be positive".to_string()))?;
        }

        // Get current wallet
        let current_wallet = WalletView::read_by_user(&mut context.pool(), user_id).await?
            .ok_or(FastJobErrorType::WalletNotFound)?;

        // Check wallet is not frozen
        if current_wallet.wallet.is_frozen {
            return Err(FastJobErrorType::InvalidField("Wallet is frozen".to_string()))?;
        }

        let previous_balance = current_wallet.wallet.available_balance.clone();

        // Create transaction record and process withdrawal atomically
        let transaction_form = TransactionInsertForm::new(
            Some(person_id), // from_user_id (user withdrawing)
            None, // to_user_id (external withdrawal)
            data.amount.clone(),
            TransactionType::UserWithdraw,
            Some(TransactionStatus::Pending),
            None, // billing_id
            None, // post_id
            format!("User withdrawal: {}", data.amount),
            None, // reference_number
            None, // metadata
            Some(Utc::now()),
            None, // completed_at
        );

        let transaction_id = TransactionView::process_transaction(&mut context.pool(), &transaction_form).await?;

        // Get updated wallet
        let updated_wallet = WalletView::read_by_user(&mut context.pool(), user_id).await?
            .unwrap().wallet;

        let total_balance = &updated_wallet.available_balance + &updated_wallet.escrow_balance 
            + &updated_wallet.pending_in - &updated_wallet.pending_out;

        Ok(Json(EnhancedWalletOperationResponse {
            wallet_id: updated_wallet.id,
            transaction_id: Some(transaction_id),
            previous_balance,
            new_balance: updated_wallet.available_balance.clone(),
            available_balance: updated_wallet.available_balance,
            escrow_balance: updated_wallet.escrow_balance,
            pending_in: updated_wallet.pending_in,
            pending_out: updated_wallet.pending_out,
            total_balance,
            transaction_amount: data.amount.clone(),
            currency: updated_wallet.currency,
            version: updated_wallet.version,
            success: true,
            message: Some("Withdrawal initiated successfully".to_string()),
        }))
    }


    /// Get transaction history with filtering
    pub async fn get_transactions(
        query: Query<GetTransactions>,
        context: Data<FastJobContext>,
        local_user_view: LocalUserView,
    ) -> FastJobResult<Json<GetTransactionsResponse>> {
        let user_id = local_user_view.local_user.id;
        let params = query.into_inner();

        let limit = params.limit.unwrap_or(50).min(100) as i64; // Max 100 transactions
        let offset = params.offset.unwrap_or(0) as i64;

        // Convert LocalUserId to PersonId for transaction queries
        let person_id = local_user_view.local_user.person_id;
        
        // Get transactions for this user from database
        let transaction_views = TransactionView::get_user_transactions(
            &mut context.pool(),
            person_id,
            Some(limit),
            Some(offset),
        ).await?;

        // Get total transaction count for pagination
        let total_count = TransactionView::get_recent_transaction_count(
            &mut context.pool(),
            person_id,
            365, // Last year
        ).await?;

        let transaction_responses = transaction_views
            .into_iter()
            .map(|tv| {
                let t = tv.transaction;
                TransactionResponse {
                    id: t.id,
                    from_user_id: None, // TODO: Convert PersonId to LocalUserId if needed
                    to_user_id: None,   // TODO: Convert PersonId to LocalUserId if needed
                    amount: t.amount,
                    transaction_type: t.transaction_type,
                    status: t.status,
                    billing_id: t.billing_id,
                    post_id: t.post_id,
                    description: t.description,
                    reference_number: t.reference_number,
                    created_at: t.created_at.to_rfc3339(),
                    completed_at: t.completed_at.map(|dt| dt.to_rfc3339()),
                }
            })
            .collect();

        let has_more = (offset + limit) < total_count;

        Ok(Json(GetTransactionsResponse {
            transactions: transaction_responses,
            total_count,
            has_more,
        }))
    }

    /// Helper: Ensure wallet exists for user
    async fn ensure_wallet_exists(
        pool: &mut lemmy_db_schema::utils::DbPool<'_>,
        user_id: LocalUserId,
    ) -> FastJobResult<()> {
        if WalletView::read_by_user(pool, user_id).await?.is_none() {
            WalletView::create_for_user(pool, user_id).await?;
        }
        Ok(())
    }
}