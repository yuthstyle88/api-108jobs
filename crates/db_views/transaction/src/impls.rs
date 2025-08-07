use crate::TransactionView;
use chrono::Utc;
use num_traits::ToPrimitive;
use diesel::{prelude::*, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{BillingId, PersonId, TransactionId},
  source::transaction::{Transaction, TransactionInsertForm, TransactionUpdateForm},
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::{
  enums::{TransactionStatus, TransactionType},
  schema::transaction,
};
use lemmy_utils::error::{FastJobErrorType, FastJobResult};

impl TransactionView {
  pub async fn read(pool: &mut DbPool<'_>, transaction_id: TransactionId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let transaction = transaction::table.find(transaction_id).first::<Transaction>(conn).await?;
    Ok(TransactionView { transaction })
  }

  pub async fn create_transaction(
    pool: &mut DbPool<'_>,
    form: &TransactionInsertForm,
  ) -> FastJobResult<Transaction> {
    let conn = &mut get_conn(pool).await?;
    
    let transaction_form = TransactionInsertForm {
      from_user_id: form.from_user_id,
      to_user_id: form.to_user_id,
      amount: form.amount.clone(),
      transaction_type: form.transaction_type,
      status: Some(form.status.unwrap_or(TransactionStatus::Pending)),
      billing_id: form.billing_id,
      post_id: form.post_id,
      description: form.description.clone(),
      reference_number: form.reference_number.clone(),
      metadata: form.metadata.clone(),
      created_at: Some(form.created_at.unwrap_or_else(|| Utc::now())),
      completed_at: form.completed_at,
    };

    diesel::insert_into(transaction::table)
      .values(&transaction_form)
      .get_result::<Transaction>(conn)
      .await
      .map_err(Into::into)
  }

  pub async fn update_transaction(
    pool: &mut DbPool<'_>,
    transaction_id: TransactionId,
    form: &TransactionUpdateForm,
  ) -> FastJobResult<Transaction> {
    let conn = &mut get_conn(pool).await?;
    
    diesel::update(transaction::table.find(transaction_id))
      .set(form)
      .get_result::<Transaction>(conn)
      .await
      .map_err(Into::into)
  }

  pub async fn complete_transaction(
    pool: &mut DbPool<'_>,
    transaction_id: TransactionId,
  ) -> FastJobResult<Transaction> {
    let form = TransactionUpdateForm {
      status: Some(TransactionStatus::Completed),
      completed_at: Some(Utc::now()),
      updated_at: Some(Utc::now()),
      ..Default::default()
    };
    
    Self::update_transaction(pool, transaction_id, &form).await
  }

  pub async fn fail_transaction(
    pool: &mut DbPool<'_>,
    transaction_id: TransactionId,
    reason: Option<String>,
  ) -> FastJobResult<Transaction> {
    let mut form = TransactionUpdateForm {
      status: Some(TransactionStatus::Failed),
      updated_at: Some(Utc::now()),
      ..Default::default()
    };

    if let Some(reason) = reason {
      let metadata = serde_json::json!({ "failure_reason": reason });
      form.metadata = Some(Some(metadata));
    }
    
    Self::update_transaction(pool, transaction_id, &form).await
  }

  /// Get transactions for a user (both sent and received)
  pub async fn get_user_transactions(
    pool: &mut DbPool<'_>,
    user_id: PersonId,
    limit: Option<i64>,
    offset: Option<i64>,
  ) -> FastJobResult<Vec<TransactionView>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit.unwrap_or(50).min(100);
    let offset = offset.unwrap_or(0);

    let transactions = transaction::table
      .filter(
        transaction::from_user_id
          .eq(user_id)
          .or(transaction::to_user_id.eq(user_id)),
      )
      .order_by(transaction::created_at.desc())
      .limit(limit)
      .offset(offset)
      .load::<Transaction>(conn)
      .await?;

    Ok(
      transactions
        .into_iter()
        .map(|transaction| TransactionView { transaction })
        .collect(),
    )
  }

  /// Get transactions for a billing record
  pub async fn get_billing_transactions(
    pool: &mut DbPool<'_>,
    billing_id: BillingId,
  ) -> FastJobResult<Vec<TransactionView>> {
    let conn = &mut get_conn(pool).await?;

    let transactions = transaction::table
      .filter(transaction::billing_id.eq(billing_id))
      .order_by(transaction::created_at.desc())
      .load::<Transaction>(conn)
      .await?;

    Ok(
      transactions
        .into_iter()
        .map(|transaction| TransactionView { transaction })
        .collect(),
    )
  }

  /// Get transaction count for user in the last N days
  pub async fn get_recent_transaction_count(
    pool: &mut DbPool<'_>,
    user_id: PersonId,
    days: i32,
  ) -> FastJobResult<i64> {
    let conn = &mut get_conn(pool).await?;
    let cutoff_date = Utc::now() - chrono::Duration::days(days as i64);

    transaction::table
      .filter(
        transaction::from_user_id
          .eq(user_id)
          .or(transaction::to_user_id.eq(user_id)),
      )
      .filter(transaction::created_at.gt(cutoff_date))
      .count()
      .get_result::<i64>(conn)
      .await
      .map_err(Into::into)
  }

  /// Process transaction with atomic wallet updates
  pub async fn process_transaction(
    pool: &mut DbPool<'_>,
    form: &TransactionInsertForm,
  ) -> FastJobResult<TransactionId> {
    let _conn = &mut get_conn(pool).await?;

    // Create transaction record first
    let transaction = Self::create_transaction(pool, form).await?;

    // Process wallet updates based on transaction type
    match form.transaction_type {
      TransactionType::AdminTopUp => {
        // AdminTopUp wallet operations are handled by the admin endpoint directly
        // to avoid circular operations. This transaction is for logging only.
      }
      TransactionType::AdminWithdraw => {
        if let Some(from_person_id) = form.from_user_id {
          use lemmy_db_views_wallet::WalletView;
          use lemmy_db_views_local_user::LocalUserView as LocalUserViewQuery;
          
          // Convert PersonId to LocalUserId for wallet operations
          let from_local_user = LocalUserViewQuery::read_person(pool, from_person_id).await
              .map_err(|_| FastJobErrorType::InvalidField("Source user not found".to_string()))?;
          let from_user_id = from_local_user.local_user.id;
          
          let amount_f64 = form.amount.to_f64().unwrap_or(0.0);
          WalletView::admin_withdraw(pool, from_user_id, amount_f64).await?;
        }
      }
      TransactionType::UserDeposit => {
        // User deposits are disabled - only admin top-ups and escrow releases are allowed
        return Err(FastJobErrorType::InvalidField("Direct user deposits not allowed - contact admin for top-up".to_string()))?;
      }
      TransactionType::JobPayment => {
        // JobPayment transactions are now only created through the billing/escrow system
        // Direct user-to-user transfers are not allowed
        return Err(FastJobErrorType::InvalidField("Direct job payments not allowed - use billing system".to_string()))?;
      }
      TransactionType::EscrowDeposit => {
        if let Some(from_person_id) = form.from_user_id {
          use lemmy_db_views_wallet::WalletView;
          use lemmy_db_views_local_user::LocalUserView as LocalUserViewQuery;
          
          // Convert PersonId to LocalUserId for wallet operations
          let from_local_user = LocalUserViewQuery::read_person(pool, from_person_id).await
              .map_err(|_| FastJobErrorType::InvalidField("Source user not found".to_string()))?;
          let from_user_id = from_local_user.local_user.id;
          
          let amount_f64 = form.amount.to_f64().unwrap_or(0.0);
          // This moves money from available balance to escrow (handled by pay_for_job)
          WalletView::pay_for_job(pool, from_user_id, amount_f64).await?;
        }
      }
      TransactionType::EscrowRelease => {
        if let Some(to_person_id) = form.to_user_id {
          use lemmy_db_views_wallet::WalletView;
          use lemmy_db_views_local_user::LocalUserView as LocalUserViewQuery;
          
          // Convert PersonId to LocalUserId for wallet operations
          let to_local_user = LocalUserViewQuery::read_person(pool, to_person_id).await
              .map_err(|_| FastJobErrorType::InvalidField("Target user not found".to_string()))?;
          let to_user_id = to_local_user.local_user.id;
          
          let amount_f64 = form.amount.to_f64().unwrap_or(0.0);
          // This moves money from escrow to freelancer's available balance
          WalletView::complete_job_payment(pool, to_user_id, amount_f64).await?;
        }
      }
      TransactionType::UserWithdraw => {
        if let Some(from_person_id) = form.from_user_id {
          use lemmy_db_views_wallet::WalletView;
          use lemmy_db_views_local_user::LocalUserView as LocalUserViewQuery;
          
          // Convert PersonId to LocalUserId for wallet operations
          let from_local_user = LocalUserViewQuery::read_person(pool, from_person_id).await
              .map_err(|_| FastJobErrorType::InvalidField("Source user not found".to_string()))?;
          let from_user_id = from_local_user.local_user.id;
          
          let amount_f64 = form.amount.to_f64().unwrap_or(0.0);
          // Deduct money from user's available balance
          WalletView::admin_withdraw(pool, from_user_id, amount_f64).await?;
        }
      }
      _ => {
        // Other transaction types can be added here
      }
    }

    // Complete the transaction
    Self::complete_transaction(pool, transaction.id).await?;

    Ok(transaction.id)
  }
}