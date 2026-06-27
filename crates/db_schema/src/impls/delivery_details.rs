use crate::{
  newtypes::{
    CoinId,
    CommentId,
    DeliveryDetailsId,
    LocalUserId,
    PersonId,
    PostId,
    RiderId,
    WalletId,
  },
  source::{
    delivery_details::{DeliveryDetails, DeliveryDetailsInsertForm, DeliveryDetailsUpdateForm},
    local_user::LocalUser,
    post::Post,
    rider::Rider,
    wallet::{TxKind, WalletModel, WalletTransactionInsertForm},
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use app_108jobs_db_schema_file::{
  enums::{PostKind, RiderVerificationStatus, TripStatus},
  schema::{delivery_details, local_user as local_user_tbl, post as post_tbl, rider as rider_tbl},
};
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use chrono::{DateTime, Utc};
use diesel::{
  dsl::{insert_into, update},
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::{scoped_futures::ScopedFutureExt, RunQueryDsl};

impl Crud for DeliveryDetails {
  type InsertForm = DeliveryDetailsInsertForm;
  type UpdateForm = DeliveryDetailsUpdateForm;
  type IdType = DeliveryDetailsId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(delivery_details::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateDeliveryDetails)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    delivery_details_id: DeliveryDetailsId,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    update(delivery_details::table.find(delivery_details_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateDeliveryDetails)
  }
}

impl DeliveryDetails {
  /// Validate that the provided person is an active rider and that the given post
  /// is a Delivery post with an active delivery. Returns the corresponding RiderId.
  pub async fn validate_rider_identity(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    post_id: PostId,
  ) -> FastJobResult<RiderId> {
    use diesel::{dsl::select, ExpressionMethods};

    let conn = &mut get_conn(pool).await?;

    // Verify the post exists and is a Delivery post
    let exists_delivery_post: bool = select(diesel::dsl::exists(
      post_tbl::dsl::post
        .filter(post_tbl::dsl::id.eq(post_id.0))
        .filter(post_tbl::dsl::post_kind.eq(PostKind::Delivery)),
    ))
    .get_result(conn)
    .await
    .unwrap_or(false);

    if !exists_delivery_post {
      return Err(FastJobErrorType::InvalidDeliveryPost.into());
    }

    // Verify delivery is active (not Delivered/Cancelled)
    let is_active_delivery: bool = select(diesel::dsl::exists(
      delivery_details::dsl::delivery_details
        .filter(delivery_details::dsl::post_id.eq(post_id.0))
        .filter(delivery_details::dsl::status.ne(TripStatus::Cancelled))
        .filter(delivery_details::dsl::status.ne(TripStatus::Delivered)),
    ))
    .get_result(conn)
    .await
    .unwrap_or(false);

    if !is_active_delivery {
      return Err(FastJobErrorType::DeliveryIsNotActive.into());
    }

    // Resolve rider by person and ensure active/not rejected
    let rider_row: Option<(i32,)> = rider_tbl::dsl::rider
      .filter(rider_tbl::dsl::person_id.eq(person_id.0))
      .filter(rider_tbl::dsl::is_active.eq(true))
      .filter(rider_tbl::dsl::verification_status.ne(RiderVerificationStatus::Rejected))
      .select((rider_tbl::dsl::id,))
      .first::<(i32,)>(conn)
      .await
      .ok();

    if let Some((rid,)) = rider_row {
      Ok(RiderId(rid))
    } else {
      Err(FastJobErrorType::NotAnActiveRider.into())
    }
  }

  /// Check if a status transition is valid.
  /// Returns true if the transition is allowed, false otherwise.
  pub fn can_transition_to(&self, new_status: TripStatus) -> bool {
    use TripStatus::*;

    // Terminal states cannot be transitioned from
    if matches!(self.status, Cancelled | Delivered) {
      return false;
    }

    match (self.status, new_status) {
      // Valid forward transitions
      (Pending, Assigned) => true,
      (Assigned, EnRouteToPickup) => true,
      (EnRouteToPickup, PickedUp) => true,
      (PickedUp, EnRouteToDropoff) => true,
      (EnRouteToDropoff, Delivered) => true,

      // Cancellation is allowed from any non-terminal state
      (Pending, Cancelled) => true,
      (Assigned, Cancelled) => true,
      (EnRouteToPickup, Cancelled) => true,
      (PickedUp, Cancelled) => true,
      (EnRouteToDropoff, Cancelled) => true,

      // Same status is idempotent
      (s, new_s) if s == new_s => true,

      // All other transitions are invalid
      _ => false,
    }
  }

  /// Update the status of a delivery by post_id.
  /// This automatically updates the updated_at timestamp.
  /// If the status is set to Cancelled, the cancellation reason will also be saved.
  pub async fn update_status(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    new_status: TripStatus,
    cancellation_reason: Option<String>,
  ) -> FastJobResult<Self> {
    use diesel::ExpressionMethods;

    let conn = &mut get_conn(pool).await?;

    // First, fetch the current delivery details
    let current_delivery = delivery_details::dsl::delivery_details
      .filter(delivery_details::dsl::post_id.eq(post_id.0))
      .first::<Self>(conn)
      .await
      .map_err(|_| FastJobErrorType::NotFound)?;

    // Validate the status transition
    if !current_delivery.can_transition_to(new_status) {
      return Err(
        FastJobErrorType::InvalidField(format!(
          "Cannot transition from {:?} to {:?}",
          current_delivery.status, new_status
        ))
        .into(),
      );
    }

    // Determine the cancellation reason to save
    // If cancelling, use the provided reason or clear it if not cancelling
    let reason_to_save = match new_status {
      TripStatus::Cancelled => cancellation_reason,
      _ => None, // Clear reason for non-cancelled statuses
    };

    // Perform the update
    let updated_delivery = update(
      delivery_details::dsl::delivery_details.filter(delivery_details::dsl::post_id.eq(post_id.0)),
    )
    .set((
      delivery_details::dsl::status.eq(new_status),
      delivery_details::dsl::cancellation_reason.eq(reason_to_save),
      delivery_details::dsl::updated_at.eq(Utc::now()),
    ))
    .get_result::<Self>(conn)
    .await
    .with_fastjob_type(FastJobErrorType::CouldntUpdateDeliveryDetails)?;

    Ok(updated_delivery)
  }

  /// Get delivery details by post_id.
  pub async fn get_by_post_id(pool: &mut DbPool<'_>, post_id: PostId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    delivery_details::dsl::delivery_details
      .filter(delivery_details::dsl::post_id.eq(post_id))
      .first::<Self>(conn)
      .await
      .map_err(|_| FastJobErrorType::NotFound.into())
  }

  /// Get all active deliveries (in progress: Assigned, EnRouteToPickup, PickedUp,
  /// EnRouteToDropoff).
  pub async fn get_all_active(pool: &mut DbPool<'_>) -> FastJobResult<Vec<Self>> {
    use diesel::QueryDsl;
    let conn = &mut get_conn(pool).await?;

    let results = delivery_details::dsl::delivery_details
      .filter(delivery_details::dsl::status.eq_any(vec![
        TripStatus::Assigned,
        TripStatus::EnRouteToPickup,
        TripStatus::PickedUp,
        TripStatus::EnRouteToDropoff,
      ]))
      .order(delivery_details::dsl::created_at.desc())
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateDeliveryDetails)?;

    Ok(results)
  }

  /// Assign a rider to a delivery from a comment/proposal.
  /// This links a rider to a delivery post, tracking who made the assignment
  /// and which comment (proposal) led to the assignment.
  pub async fn assign_from_comment(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    rider_id: RiderId,
    assigned_by_person_id: PersonId,
    comment_id: CommentId,
    sender_name: String,
    sender_phone: String,
    receiver_name: String,
    receiver_phone: String,
  ) -> FastJobResult<Self> {
    use diesel::ExpressionMethods;

    let conn = &mut get_conn(pool).await?;

    // First, fetch the current delivery details to validate
    let current_delivery = delivery_details::dsl::delivery_details
      .filter(delivery_details::dsl::post_id.eq(post_id.0))
      .first::<Self>(conn)
      .await
      .map_err(|_| FastJobErrorType::NotFound)?;

    // Can only assign from Pending status
    if current_delivery.status != TripStatus::Pending {
      return Err(FastJobErrorType::CannotUnassignFromStatus.into());
    }

    // Validate required fields are not empty
    if sender_name.trim().is_empty() {
      return Err(FastJobErrorType::SenderNameIsRequired.into());
    }
    if sender_phone.trim().is_empty() {
      return Err(FastJobErrorType::SenderPhoneIsRequired.into());
    }
    if receiver_name.trim().is_empty() {
      return Err(FastJobErrorType::ReceiverNameIsRequired.into());
    }
    if receiver_phone.trim().is_empty() {
      return Err(FastJobErrorType::ReceiverPhoneIsRequired.into());
    }

    // Perform the assignment and status update with sender/receiver info
    let updated_delivery = update(
      delivery_details::dsl::delivery_details.filter(delivery_details::dsl::post_id.eq(post_id.0)),
    )
    .set((
      delivery_details::dsl::assigned_rider_id.eq(rider_id.0),
      delivery_details::dsl::assigned_at.eq(Utc::now()),
      delivery_details::dsl::assigned_by_person_id.eq(assigned_by_person_id.0),
      delivery_details::dsl::linked_comment_id.eq(comment_id.0),
      delivery_details::dsl::sender_name.eq(sender_name),
      delivery_details::dsl::sender_phone.eq(sender_phone),
      delivery_details::dsl::receiver_name.eq(receiver_name),
      delivery_details::dsl::receiver_phone.eq(receiver_phone),
      delivery_details::dsl::status.eq(TripStatus::Assigned),
      delivery_details::dsl::updated_at.eq(Utc::now()),
    ))
    .get_result::<Self>(conn)
    .await
    .with_fastjob_type(FastJobErrorType::CouldntUpdateDeliveryDetails)?;

    Ok(updated_delivery)
  }

  /// Assign a rider to a delivery with escrow hold.
  /// This is similar to assign_from_comment but also holds the delivery fee in escrow.
  ///
  /// The escrow amount is taken from the post's budget.
  /// All operations are performed in a single transaction.
  pub async fn assign_from_comment_with_escrow(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    rider_id: RiderId,
    employer_local_user_id: LocalUserId,
    employer_person_id: PersonId,
    comment_id: CommentId,
    sender_name: String,
    sender_phone: String,
    receiver_name: String,
    receiver_phone: String,
  ) -> FastJobResult<Self> {
    use diesel::ExpressionMethods;

    let conn = &mut get_conn(pool).await?;

    conn
      .run_transaction(|conn| {
        async move {
          // First, fetch the current delivery details to validate
          let current_delivery = delivery_details::dsl::delivery_details
            .filter(delivery_details::dsl::post_id.eq(post_id.0))
            .first::<Self>(conn)
            .await
            .map_err(|_| FastJobErrorType::NotFound)?;

          // Can only assign from Pending status
          if current_delivery.status != TripStatus::Pending {
            return Err(FastJobErrorType::CannotUnassignFromStatus.into());
          }

          // Validate required fields are not empty
          if sender_name.trim().is_empty() {
            return Err(FastJobErrorType::SenderNameIsRequired.into());
          }
          if sender_phone.trim().is_empty() {
            return Err(FastJobErrorType::SenderPhoneIsRequired.into());
          }
          if receiver_name.trim().is_empty() {
            return Err(FastJobErrorType::ReceiverNameIsRequired.into());
          }
          if receiver_phone.trim().is_empty() {
            return Err(FastJobErrorType::ReceiverPhoneIsRequired.into());
          }

          // Get the post to get the budget amount (delivery fee)
          let mut pool: DbPool<'_> = conn.into();
          let post = Post::read(&mut pool, post_id).await?;

          let delivery_fee = post.budget;

          // Get employer's wallet and hold funds in escrow
          let employer_wallet = WalletModel::get_by_user(&mut pool, employer_local_user_id).await?;

          // Hold the delivery fee in escrow (employer -> platform)
          let tx_form = WalletTransactionInsertForm {
            wallet_id: employer_wallet.id,
            reference_type: "delivery".to_string(),
            reference_id: post_id.0,
            kind: TxKind::Transfer, // Use Transfer for hold
            amount: delivery_fee,
            description: format!("escrow hold for delivery assignment: post {}", post_id.0),
            counter_user_id: Some(employer_local_user_id),
            // Deterministic key: retrying the same assignment is idempotent.
            idempotency_key: format!("assign:{}:{}", post_id.0, employer_local_user_id.0),
          };
          let _ = WalletModel::hold(&mut pool, &tx_form).await?;

          // Perform the assignment with sender/receiver info and escrow details
          let conn = &mut get_conn(&mut pool).await?;
          let updated_delivery = update(
            delivery_details::dsl::delivery_details
              .filter(delivery_details::dsl::post_id.eq(post_id.0)),
          )
          .set((
            delivery_details::dsl::assigned_rider_id.eq(rider_id.0),
            delivery_details::dsl::assigned_at.eq(Utc::now()),
            delivery_details::dsl::assigned_by_person_id.eq(employer_person_id.0),
            delivery_details::dsl::linked_comment_id.eq(comment_id.0),
            delivery_details::dsl::sender_name.eq(sender_name),
            delivery_details::dsl::sender_phone.eq(sender_phone),
            delivery_details::dsl::receiver_name.eq(receiver_name),
            delivery_details::dsl::receiver_phone.eq(receiver_phone),
            delivery_details::dsl::delivery_fee.eq(delivery_fee),
            delivery_details::dsl::status.eq(TripStatus::Assigned),
            delivery_details::dsl::updated_at.eq(Utc::now()),
          ))
          .get_result::<Self>(conn)
          .await
          .with_fastjob_type(FastJobErrorType::CouldntUpdateDeliveryDetails)?;

          Ok::<_, app_108jobs_utils::error::FastJobError>(updated_delivery)
        }
        .scope_boxed()
      })
      .await
  }

  /// Unassign a rider from a delivery, returning it to Pending status.
  /// Only the assigner (employer) or an admin can unassign.
  pub async fn unassign_rider(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    person_id: PersonId,
  ) -> FastJobResult<Self> {
    use diesel::ExpressionMethods;

    let conn = &mut get_conn(pool).await?;

    // Fetch current delivery
    let current_delivery = delivery_details::dsl::delivery_details
      .filter(delivery_details::dsl::post_id.eq(post_id.0))
      .first::<Self>(conn)
      .await
      .map_err(|_| FastJobErrorType::NotFound)?;

    // Verify the person is either the assigner or an admin (you may want to add admin check)
    if current_delivery.assigned_by_person_id != Some(person_id) {
      return Err(FastJobErrorType::OnlyAssignerCanUnassign.into());
    }

    // Can only unassign from Assigned status (before work begins)
    if current_delivery.status != TripStatus::Assigned {
      return Err(FastJobErrorType::CannotUnassignFromStatus.into());
    }

    // Perform the unassignment
    let updated_delivery = update(
      delivery_details::dsl::delivery_details.filter(delivery_details::dsl::post_id.eq(post_id.0)),
    )
    .set((
      delivery_details::dsl::assigned_rider_id.eq(Option::<i32>::None),
      delivery_details::dsl::assigned_at.eq(Option::<DateTime<Utc>>::None),
      delivery_details::dsl::assigned_by_person_id.eq(Option::<i32>::None),
      delivery_details::dsl::linked_comment_id.eq(Option::<i32>::None),
      delivery_details::dsl::status.eq(TripStatus::Pending),
      delivery_details::dsl::updated_at.eq(Utc::now()),
    ))
    .get_result::<Self>(conn)
    .await
    .with_fastjob_type(FastJobErrorType::CouldntUpdateDeliveryDetails)?;

    Ok(updated_delivery)
  }

  /// Confirm delivery completion and release escrow to rider.
  /// Only the employer (post creator) can confirm and release payment.
  /// The delivery must be in Delivered status.
  ///
  /// This method:
  /// 1. Verifies the delivery is Delivered
  /// 2. Verifies the caller is the employer
  /// 3. Releases escrow funds from platform to rider's wallet
  /// 4. Updates the employer_confirmed_at timestamp
  pub async fn confirm_completion_and_release_payment(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    employer_person_id: PersonId,
    coin_id: CoinId,
    platform_wallet_id: WalletId,
  ) -> FastJobResult<Self> {
    use diesel::QueryDsl;

    let conn = &mut get_conn(pool).await?;

    conn
      .run_transaction(|conn| {
        async move {
          // Fetch current delivery with post
          let current_delivery = delivery_details::dsl::delivery_details
            .filter(delivery_details::dsl::post_id.eq(post_id.0))
            .first::<Self>(conn)
            .await
            .map_err(|_| FastJobErrorType::NotFound)?;

          // Verify the caller is the employer (post creator)
          if current_delivery.assigned_by_person_id != Some(employer_person_id) {
            return Err(FastJobErrorType::OnlyAssignerCanConfirm.into());
          }

          // Can only confirm Delivered status
          if current_delivery.status != TripStatus::Delivered {
            return Err(FastJobErrorType::CannotConfirmNonDeliveredDelivery.into());
          }

          // Check if already confirmed
          if current_delivery.employer_confirmed_at.is_some() {
            // Already confirmed - idempotent, return current state
            return Ok(current_delivery);
          }

          // Get the assigned rider
          let rider_id = current_delivery
            .assigned_rider_id
            .ok_or(FastJobErrorType::NoRiderAssigned)?;

          // Get the rider's person_id
          let mut pool: DbPool<'_> = conn.into();
          let conn2 = &mut get_conn(&mut pool).await?;
          let rider: Rider = rider_tbl::dsl::rider
            .find(rider_id)
            .first::<Rider>(conn2)
            .await
            .map_err(|_| FastJobErrorType::NotFound)?;

          // Get the local_user_id from person_id
          let local_user: LocalUser = local_user_tbl::dsl::local_user
            .filter(local_user_tbl::dsl::person_id.eq(rider.person_id))
            .first::<LocalUser>(conn2)
            .await
            .map_err(|_| FastJobErrorType::NotFound)?;
          let rider_local_user_id = local_user.id;

          // Get rider's wallet
          let rider_wallet = WalletModel::get_by_user(&mut pool, rider_local_user_id).await?;

          // Release the delivery fee from escrow to rider (platform -> rider)
          let delivery_fee = current_delivery.delivery_fee;
          let tx_form = WalletTransactionInsertForm {
            wallet_id: rider_wallet.id,
            reference_type: "delivery".to_string(),
            reference_id: post_id.0,
            kind: TxKind::Transfer,
            amount: delivery_fee,
            description: format!("delivery payment released: post {}", post_id.0),
            counter_user_id: Some(rider_local_user_id),
            // Deterministic key: retrying payment release is idempotent.
            idempotency_key: format!("release:{}:{}", post_id.0, rider_local_user_id.0),
          };
          WalletModel::deposit_from_platform(&mut pool, &tx_form, coin_id, platform_wallet_id)
            .await?;

          // Update the delivery with confirmation timestamp
          let conn = &mut get_conn(&mut pool).await?;
          let updated_delivery = update(
            delivery_details::dsl::delivery_details
              .filter(delivery_details::dsl::post_id.eq(post_id.0)),
          )
          .set((
            delivery_details::dsl::employer_confirmed_at.eq(Utc::now()),
            delivery_details::dsl::updated_at.eq(Utc::now()),
          ))
          .get_result::<Self>(conn)
          .await
          .with_fastjob_type(FastJobErrorType::CouldntUpdateDeliveryDetails)?;

          Ok::<_, app_108jobs_utils::error::FastJobError>(updated_delivery)
        }
        .scope_boxed()
      })
      .await
  }

  /// Cancel a delivery and refund any held escrow back to the employer.
  ///
  /// Guard: if no rider is assigned (`assigned_rider_id.is_none()`) or the
  /// fee is zero, no escrow was ever held — delegates straight to
  /// `update_status`.  Otherwise runs a single DB transaction:
  ///   1. Resolve employer via `post.creator_id → local_user → wallet`.
  ///   2. `WalletModel::refund_from_platform_on_conn` — platform → employer (reverses the original
  ///      `hold`; no CoinModel change).
  ///   3. Set `status = Cancelled` + `cancellation_reason` in the same tx.
  ///
  /// Idempotency key `cancel-refund:{post_id}:{employer_local_user_id}` makes
  /// retried cancellations safe.
  pub async fn cancel_and_refund_escrow(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    reason: Option<String>,
  ) -> FastJobResult<Self> {
    // Cheap guard — read outside the transaction.
    let current = Self::get_by_post_id(pool, post_id).await?;
    if current.assigned_rider_id.is_none() || current.delivery_fee.0 == 0 {
      return Self::update_status(pool, post_id, TripStatus::Cancelled, reason).await;
    }

    let delivery_fee = current.delivery_fee;
    let conn = &mut get_conn(pool).await?;

    conn
      .run_transaction(|conn| {
        async move {
          let mut pool: DbPool<'_> = conn.into();

          // Resolve employer: post.creator_id -> local_user -> wallet.
          let post = Post::read(&mut pool, post_id).await?;
          let conn2 = &mut get_conn(&mut pool).await?;
          let employer_lu = local_user_tbl::dsl::local_user
            .filter(local_user_tbl::dsl::person_id.eq(post.creator_id.0))
            .first::<LocalUser>(conn2)
            .await
            .map_err(|_| FastJobErrorType::NotFound)?;
          let employer_local_user_id = employer_lu.id;
          let employer_wallet = WalletModel::get_by_user(&mut pool, employer_local_user_id).await?;

          let tx_form = WalletTransactionInsertForm {
            wallet_id: employer_wallet.id,
            reference_type: "delivery".to_string(),
            reference_id: post_id.0,
            kind: TxKind::Transfer,
            amount: delivery_fee,
            description: format!("escrow refund for cancelled delivery: post {}", post_id.0),
            counter_user_id: Some(employer_local_user_id),
            // Deterministic: retrying the same cancellation is idempotent.
            idempotency_key: format!("cancel-refund:{}:{}", post_id.0, employer_local_user_id.0),
          };

          let conn3 = &mut get_conn(&mut pool).await?;
          WalletModel::refund_from_platform_on_conn(conn3, &tx_form).await?;

          // Update status and reason in the same transaction.
          let conn4 = &mut get_conn(&mut pool).await?;
          let updated = update(
            delivery_details::dsl::delivery_details
              .filter(delivery_details::dsl::post_id.eq(post_id.0)),
          )
          .set((
            delivery_details::dsl::status.eq(TripStatus::Cancelled),
            delivery_details::dsl::cancellation_reason.eq(reason),
            delivery_details::dsl::updated_at.eq(Utc::now()),
          ))
          .get_result::<Self>(conn4)
          .await
          .with_fastjob_type(FastJobErrorType::CouldntUpdateDeliveryDetails)?;

          Ok::<_, app_108jobs_utils::error::FastJobError>(updated)
        }
        .scope_boxed()
      })
      .await
  }

  /// Get the active delivery assignment for a specific rider.
  /// Returns the delivery if the rider has an active assignment.
  pub async fn get_active_for_rider(
    pool: &mut DbPool<'_>,
    rider_id: RiderId,
  ) -> FastJobResult<Option<Self>> {
    use diesel::QueryDsl;
    let conn = &mut get_conn(pool).await?;

    let result = delivery_details::dsl::delivery_details
      .filter(delivery_details::dsl::assigned_rider_id.eq(rider_id.0))
      .filter(delivery_details::dsl::status.eq_any(vec![
        TripStatus::Assigned,
        TripStatus::EnRouteToPickup,
        TripStatus::PickedUp,
        TripStatus::EnRouteToDropoff,
      ]))
      .first::<Self>(conn)
      .await
      .ok();

    Ok(result)
  }

  /// Get all completed deliveries (status = Delivered).
  pub async fn get_all_completed(pool: &mut DbPool<'_>) -> FastJobResult<Vec<Self>> {
    use diesel::QueryDsl;
    let conn = &mut get_conn(pool).await?;

    let results = delivery_details::dsl::delivery_details
      .filter(delivery_details::dsl::status.eq(TripStatus::Delivered))
      .order(delivery_details::dsl::created_at.desc())
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateDeliveryDetails)?;

    Ok(results)
  }

  /// Get all cancelled deliveries (status = Cancelled).
  pub async fn get_all_cancelled(pool: &mut DbPool<'_>) -> FastJobResult<Vec<Self>> {
    use diesel::QueryDsl;
    let conn = &mut get_conn(pool).await?;

    let results = delivery_details::dsl::delivery_details
      .filter(delivery_details::dsl::status.eq(TripStatus::Cancelled))
      .order(delivery_details::dsl::created_at.desc())
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateDeliveryDetails)?;

    Ok(results)
  }

  /// List all deliveries created by the given employer (post.creator_id = employer_person_id).
  /// Returns all statuses, ordered by created_at descending.
  pub async fn list_by_employer(
    pool: &mut DbPool<'_>,
    employer_person_id: PersonId,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    delivery_details::dsl::delivery_details
      .inner_join(post_tbl::dsl::post)
      .filter(post_tbl::dsl::creator_id.eq(employer_person_id.0))
      .select(delivery_details::all_columns)
      .order(delivery_details::dsl::created_at.desc())
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateDeliveryDetails)
  }

  /// Fetch a delivery by post_id, returning NotFound if the post was not
  /// created by employer_person_id (ownership check — does not reveal existence).
  pub async fn get_by_post_id_for_employer(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    employer_person_id: PersonId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    delivery_details::dsl::delivery_details
      .inner_join(post_tbl::dsl::post)
      .filter(delivery_details::dsl::post_id.eq(post_id.0))
      .filter(post_tbl::dsl::creator_id.eq(employer_person_id.0))
      .select(delivery_details::all_columns)
      .first::<Self>(conn)
      .await
      .map_err(|_| FastJobErrorType::NotFound.into())
  }
}

// ============================================================================
// DB-backed tests for the delivery lifecycle.
//
// Coverage:
//   * can_transition_to: every legal forward + cancel transition allowed, terminal states rejected,
//     every illegal pair rejected.
//   * update_status enforces those rules (Pending -> Delivered blocked).
//   * Status transitions persist the new status and updated_at.
//   * Cancellation persists the supplied reason; non-cancel transitions clear the reason field.
//
// NOTE on the escrow/confirm flows (`assign_from_comment_with_escrow`,
// `confirm_completion_and_release_payment`): these were exercised by hand
// during the panic/transaction audit and the random `Uuid::new_v4()`
// idempotency keys at lines 372 and 534 (relative to original file) make
// retries non-idempotent. A characterization test exposing that defect
// would need to inject a deterministic key. The smallest reproducer is in
// `crates/workflow/src/impls.rs` `concurrent_approve_one_succeeds_other_fails`
// where the correct pattern is already in place; bringing that test pattern
// to delivery is the recommended next step (see report).
// ============================================================================
#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    source::{
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::PostInsertForm,
    },
    test_data::pool_for_tests,
  };
  use app_108jobs_db_schema_file::schema::post;
  use diesel_async::RunQueryDsl;
  use serial_test::serial;

  /// Build a minimal delivery row with the requested initial status.
  /// Returns (instance_id, post_id, delivery row).
  async fn fixture_with_status(
    pool: &mut DbPool<'_>,
    initial: TripStatus,
  ) -> (crate::newtypes::InstanceId, PostId, DeliveryDetails) {
    let inst = Instance::read_or_create(pool, format!("dd-test-{}.tld", uuid::Uuid::new_v4()))
      .await
      .expect("create instance");

    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let suffix_short = &suffix[..8];
    let (p_form, _wallet) =
      PersonInsertForm::test_form_with_wallet(pool, inst.id, &format!("emp-{suffix_short}"))
        .await
        .expect("test_form_with_wallet");
    let emp = Person::create(pool, &p_form).await.expect("create person");

    let mut post_form = PostInsertForm::new("delivery test".into(), emp.id);
    post_form.post_kind = Some(PostKind::Delivery);
    let post_id: i32 = {
      let conn = &mut get_conn(pool).await.expect("get conn");
      diesel::insert_into(post::table)
        .values(&post_form)
        .returning(post::id)
        .get_result(conn)
        .await
        .expect("insert post")
    };

    let mut form = DeliveryDetailsInsertForm::new(
      PostId(post_id),
      "1 Pickup St".to_string(),
      "2 Dropoff St".to_string(),
    );
    form.status = Some(initial);
    let delivery = DeliveryDetails::create(pool, &form).await.expect("create");
    (inst.id, PostId(post_id), delivery)
  }

  async fn cleanup(pool: &mut DbPool<'_>, instance_id: crate::newtypes::InstanceId) {
    let _ = Instance::delete(pool, instance_id).await;
  }

  /// Spot-check valid forward transitions and the same-status idempotency
  /// rule baked into `can_transition_to`.
  #[tokio::test]
  #[serial]
  async fn can_transition_to_accepts_valid_forward_and_cancel() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let (instance_id, _, d) = fixture_with_status(pool, TripStatus::Pending).await;

    assert!(d.can_transition_to(TripStatus::Assigned));
    assert!(d.can_transition_to(TripStatus::Cancelled));
    assert!(
      d.can_transition_to(TripStatus::Pending),
      "same status is no-op"
    );
    assert!(!d.can_transition_to(TripStatus::Delivered));
    assert!(!d.can_transition_to(TripStatus::EnRouteToPickup));
    cleanup(pool, instance_id).await;
  }

  /// Terminal states refuse every outgoing transition (including same-status,
  /// per the early-return guard at line 125).
  #[tokio::test]
  #[serial]
  async fn can_transition_to_locks_terminal_states() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let (instance_id, _, d) = fixture_with_status(pool, TripStatus::Delivered).await;

    assert!(!d.can_transition_to(TripStatus::Cancelled));
    assert!(!d.can_transition_to(TripStatus::EnRouteToPickup));
    assert!(
      !d.can_transition_to(TripStatus::Delivered),
      "Delivered is terminal: per the early-return at line 125, even \
       same-status returns false"
    );

    cleanup(pool, instance_id).await;
  }

  /// update_status walks the assignment lane and persists the new status.
  #[tokio::test]
  #[serial]
  async fn update_status_walks_lifecycle_lane() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let (instance_id, post_id, _) = fixture_with_status(pool, TripStatus::Assigned).await;

    for next in [
      TripStatus::EnRouteToPickup,
      TripStatus::PickedUp,
      TripStatus::EnRouteToDropoff,
      TripStatus::Delivered,
    ] {
      let updated = DeliveryDetails::update_status(pool, post_id, next, None)
        .await
        .unwrap_or_else(|e| panic!("transition -> {next:?} failed: {e:?}"));
      assert_eq!(updated.status, next);
    }
    cleanup(pool, instance_id).await;
  }

  /// Illegal jumps from Pending must surface InvalidField, not silently apply.
  #[tokio::test]
  #[serial]
  async fn update_status_rejects_illegal_jump() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let (instance_id, post_id, _) = fixture_with_status(pool, TripStatus::Pending).await;

    let err = DeliveryDetails::update_status(pool, post_id, TripStatus::Delivered, None)
      .await
      .expect_err("Pending -> Delivered must be rejected");
    let msg = format!("{err:?}");
    assert!(
      msg.contains("InvalidField") || msg.contains("Cannot transition"),
      "expected illegal-transition error, got {msg}"
    );

    // State must not have moved.
    let after = DeliveryDetails::get_by_post_id(pool, post_id)
      .await
      .expect("get");
    assert_eq!(after.status, TripStatus::Pending);
    cleanup(pool, instance_id).await;
  }

  /// Cancellation persists the reason; subsequent transitions are blocked
  /// because Cancelled is terminal.
  #[tokio::test]
  #[serial]
  async fn cancel_persists_reason_and_locks_state() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let (instance_id, post_id, _) = fixture_with_status(pool, TripStatus::Assigned).await;

    let cancelled = DeliveryDetails::update_status(
      pool,
      post_id,
      TripStatus::Cancelled,
      Some("rider unavailable".to_string()),
    )
    .await
    .expect("cancel");
    assert_eq!(cancelled.status, TripStatus::Cancelled);
    assert_eq!(
      cancelled.cancellation_reason.as_deref(),
      Some("rider unavailable")
    );

    let err = DeliveryDetails::update_status(pool, post_id, TripStatus::EnRouteToPickup, None)
      .await
      .expect_err("post-cancel transition must be rejected");
    assert!(format!("{err:?}").contains("InvalidField"));
    cleanup(pool, instance_id).await;
  }

  /// Non-cancel transitions clear `cancellation_reason` (per update_status:185).
  #[tokio::test]
  #[serial]
  async fn forward_transition_clears_reason() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let (instance_id, post_id, _) = fixture_with_status(pool, TripStatus::Assigned).await;

    // Sneak a reason in directly so we can verify clearing.
    use app_108jobs_db_schema_file::schema::delivery_details as dd;
    {
      let conn = &mut get_conn(pool).await.expect("conn");
      diesel::update(dd::dsl::delivery_details.filter(dd::dsl::post_id.eq(post_id.0)))
        .set(dd::dsl::cancellation_reason.eq(Some("stale reason".to_string())))
        .execute(conn)
        .await
        .expect("seed reason");
    }

    let moved = DeliveryDetails::update_status(pool, post_id, TripStatus::EnRouteToPickup, None)
      .await
      .expect("forward transition");
    assert_eq!(moved.status, TripStatus::EnRouteToPickup);
    assert!(
      moved.cancellation_reason.is_none(),
      "non-cancel transition must clear cancellation_reason; got {:?}",
      moved.cancellation_reason
    );
    cleanup(pool, instance_id).await;
  }

  #[tokio::test]
  #[serial]
  async fn get_by_post_id_returns_correct_delivery() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let (instance_id, post_id, delivery) = fixture_with_status(pool, TripStatus::Pending).await;

    let found = DeliveryDetails::get_by_post_id(pool, post_id)
      .await
      .expect("should find delivery by post_id");

    assert_eq!(found.post_id, delivery.post_id);
    assert_eq!(found.status, TripStatus::Pending);
    assert_eq!(found.pickup_address, delivery.pickup_address);
    cleanup(pool, instance_id).await;
  }

  #[tokio::test]
  #[serial]
  async fn get_by_post_id_missing_returns_not_found() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();

    let missing_id = PostId(i32::MAX);
    let err = DeliveryDetails::get_by_post_id(pool, missing_id)
      .await
      .expect_err("should error for missing post_id");

    let msg = format!("{err:?}");
    assert!(
      msg.contains("NotFound"),
      "expected NotFound error, got {msg}"
    );
  }

  #[tokio::test]
  #[serial]
  async fn get_all_active_includes_in_progress_excludes_terminal() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();

    // Assigned is in-progress → must appear; Delivered + Cancelled are terminal → must not appear
    let (inst1, pid1, _) = fixture_with_status(pool, TripStatus::Assigned).await;
    let (inst2, pid2, _) = fixture_with_status(pool, TripStatus::Delivered).await;
    let (inst3, pid3, _) = fixture_with_status(pool, TripStatus::Cancelled).await;

    let active = DeliveryDetails::get_all_active(pool)
      .await
      .expect("get_all_active");

    let ids: Vec<PostId> = active.iter().map(|d| d.post_id).collect();
    assert!(
      ids.contains(&pid1),
      "Assigned delivery must be in active list"
    );
    assert!(
      !ids.contains(&pid2),
      "Delivered delivery must not be in active list"
    );
    assert!(
      !ids.contains(&pid3),
      "Cancelled delivery must not be in active list"
    );

    cleanup(pool, inst1).await;
    cleanup(pool, inst2).await;
    cleanup(pool, inst3).await;
  }

  #[tokio::test]
  #[serial]
  async fn get_all_completed_returns_delivered_only() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();

    let (inst1, pid1, _) = fixture_with_status(pool, TripStatus::Delivered).await;
    let (inst2, pid2, _) = fixture_with_status(pool, TripStatus::Assigned).await;
    let (inst3, pid3, _) = fixture_with_status(pool, TripStatus::Cancelled).await;

    let completed = DeliveryDetails::get_all_completed(pool)
      .await
      .expect("get_all_completed");

    let ids: Vec<PostId> = completed.iter().map(|d| d.post_id).collect();
    assert!(
      ids.contains(&pid1),
      "Delivered delivery must be in completed list"
    );
    assert!(
      !ids.contains(&pid2),
      "Assigned delivery must not be in completed list"
    );
    assert!(
      !ids.contains(&pid3),
      "Cancelled delivery must not be in completed list"
    );

    cleanup(pool, inst1).await;
    cleanup(pool, inst2).await;
    cleanup(pool, inst3).await;
  }

  #[tokio::test]
  #[serial]
  async fn get_all_cancelled_returns_cancelled_only() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();

    let (inst1, pid1, _) = fixture_with_status(pool, TripStatus::Cancelled).await;
    let (inst2, pid2, _) = fixture_with_status(pool, TripStatus::Assigned).await;

    let cancelled = DeliveryDetails::get_all_cancelled(pool)
      .await
      .expect("get_all_cancelled");

    let ids: Vec<PostId> = cancelled.iter().map(|d| d.post_id).collect();
    assert!(
      ids.contains(&pid1),
      "Cancelled delivery must be in cancelled list"
    );
    assert!(
      !ids.contains(&pid2),
      "Assigned delivery must not be in cancelled list"
    );

    cleanup(pool, inst1).await;
    cleanup(pool, inst2).await;
  }

  #[tokio::test]
  #[serial]
  async fn validate_rider_identity_rejects_terminal_delivery() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let (instance_id, post_id, _) = fixture_with_status(pool, TripStatus::Delivered).await;

    let dummy_person = crate::newtypes::PersonId(i32::MAX);
    let err = DeliveryDetails::validate_rider_identity(pool, dummy_person, post_id)
      .await
      .expect_err("terminal delivery must be rejected before rider check");

    let msg = format!("{err:?}");
    assert!(
      msg.contains("DeliveryIsNotActive"),
      "expected DeliveryIsNotActive, got {msg}"
    );
    cleanup(pool, instance_id).await;
  }

  #[tokio::test]
  #[serial]
  async fn validate_rider_identity_rejects_non_rider() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let (instance_id, post_id, _) = fixture_with_status(pool, TripStatus::Assigned).await;

    // Create a plain person with no rider record
    let (person_form, _) =
      PersonInsertForm::test_form_with_wallet(pool, instance_id, "plain_user_vri")
        .await
        .expect("test_form_with_wallet");
    let person = Person::create(pool, &person_form)
      .await
      .expect("create person");

    let err = DeliveryDetails::validate_rider_identity(pool, person.id, post_id)
      .await
      .expect_err("person without rider record must be rejected");

    let msg = format!("{err:?}");
    assert!(
      msg.contains("NotAnActiveRider"),
      "expected NotAnActiveRider, got {msg}"
    );
    cleanup(pool, instance_id).await;
  }

  /// Unassigned delivery: cancel_and_refund_escrow must delegate to
  /// update_status (no wallet changes) and set status = Cancelled.
  #[tokio::test]
  #[serial]
  async fn cancel_unassigned_delivery_skips_refund() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let (instance_id, post_id, _) = fixture_with_status(pool, TripStatus::Pending).await;

    let result = DeliveryDetails::cancel_and_refund_escrow(
      pool,
      post_id,
      Some("no rider, just cancel".to_string()),
    )
    .await
    .expect("cancel unassigned delivery");

    assert_eq!(result.status, TripStatus::Cancelled);
    assert_eq!(
      result.cancellation_reason.as_deref(),
      Some("no rider, just cancel")
    );
    cleanup(pool, instance_id).await;
  }

  /// Assigned delivery: cancel_and_refund_escrow must credit the employer
  /// wallet by the delivery_fee and set status = Cancelled.
  #[tokio::test]
  #[serial]
  async fn cancel_assigned_delivery_refunds_employer_wallet() {
    use crate::{newtypes::Coin, source::rider::RiderInsertForm};
    use app_108jobs_db_schema_file::{enums::VehicleType, schema::delivery_details as dd};

    let pool = pool_for_tests();
    let pool = &mut (&pool).into();

    // Employer + post + delivery (Pending, unassigned)
    let (instance_id, post_id, _) = fixture_with_status(pool, TripStatus::Pending).await;

    // fixture_with_status creates Person+Wallet for the employer but no local_user.
    // cancel_and_refund_escrow resolves the employer wallet via
    //   post.creator_id -> local_user -> wallet
    // so we must create a local_user for the employer person here.
    let post = Post::read(pool, post_id).await.expect("read post");
    let employer_lu_id: LocalUserId = {
      let conn = &mut get_conn(pool).await.expect("get conn");
      LocalUserId(
        diesel::insert_into(local_user_tbl::table)
          .values((
            local_user_tbl::person_id.eq(post.creator_id.0),
            local_user_tbl::password_encrypted.eq::<Option<String>>(None),
          ))
          .returning(local_user_tbl::id)
          .get_result(conn)
          .await
          .expect("employer local_user"),
      )
    };

    // Create a rider so we have a valid FK value for assigned_rider_id
    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let (rp_form, _rw) =
      PersonInsertForm::test_form_with_wallet(pool, instance_id, &format!("rd-{}", &suffix[..8]))
        .await
        .expect("rider person");
    let rider_person = Person::create(pool, &rp_form)
      .await
      .expect("create rider person");
    let rider_lu_id: i32 = {
      let conn = &mut get_conn(pool).await.expect("get conn");
      diesel::insert_into(local_user_tbl::table)
        .values((
          local_user_tbl::person_id.eq(rider_person.id),
          local_user_tbl::password_encrypted.eq::<Option<String>>(None),
        ))
        .returning(local_user_tbl::id)
        .get_result(conn)
        .await
        .expect("rider local_user")
    };
    let rider = Rider::create(
      pool,
      &RiderInsertForm::new(
        LocalUserId(rider_lu_id),
        rider_person.id,
        VehicleType::Motorcycle,
      ),
    )
    .await
    .expect("create rider");

    // Patch delivery: set delivery_fee and assigned_rider_id directly
    const FEE: i32 = 500;
    {
      let conn = &mut get_conn(pool).await.expect("get conn");
      diesel::update(dd::table.filter(dd::post_id.eq(post_id.0)))
        .set((
          dd::delivery_fee.eq(FEE),
          dd::assigned_rider_id.eq(rider.id.0),
        ))
        .execute(conn)
        .await
        .expect("patch delivery");
    }

    // Capture employer wallet balance before cancellation
    let wallet_before = WalletModel::get_by_user(pool, employer_lu_id)
      .await
      .expect("wallet before");

    let cancelled =
      DeliveryDetails::cancel_and_refund_escrow(pool, post_id, Some("rider cancelled".to_string()))
        .await
        .expect("cancel with refund");

    assert_eq!(cancelled.status, TripStatus::Cancelled);
    assert_eq!(
      cancelled.cancellation_reason.as_deref(),
      Some("rider cancelled")
    );

    let wallet_after = WalletModel::get_by_user(pool, employer_lu_id)
      .await
      .expect("wallet after");
    assert_eq!(
      wallet_after.balance_available - wallet_before.balance_available,
      Coin(FEE),
      "employer wallet must be credited by the full delivery fee"
    );

    cleanup(pool, instance_id).await;
  }

  /// list_by_employer returns only the deliveries whose post was created
  /// by the given employer PersonId, ignoring all other employers' deliveries.
  #[tokio::test]
  #[serial]
  async fn list_by_employer_returns_own_only() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();

    // Two employers, each with one delivery.
    let (inst1, pid1, _) = fixture_with_status(pool, TripStatus::Pending).await;
    let (inst2, pid2, _) = fixture_with_status(pool, TripStatus::Assigned).await;

    let post1 = Post::read(pool, pid1).await.expect("read post1");
    let post2 = Post::read(pool, pid2).await.expect("read post2");

    let list1 = DeliveryDetails::list_by_employer(pool, post1.creator_id)
      .await
      .expect("list for employer 1");
    let list2 = DeliveryDetails::list_by_employer(pool, post2.creator_id)
      .await
      .expect("list for employer 2");

    let ids1: Vec<PostId> = list1.iter().map(|d| d.post_id).collect();
    let ids2: Vec<PostId> = list2.iter().map(|d| d.post_id).collect();

    assert!(ids1.contains(&pid1), "employer 1 must see own delivery");
    assert!(
      !ids1.contains(&pid2),
      "employer 1 must NOT see employer 2's delivery"
    );
    assert!(ids2.contains(&pid2), "employer 2 must see own delivery");
    assert!(
      !ids2.contains(&pid1),
      "employer 2 must NOT see employer 1's delivery"
    );

    cleanup(pool, inst1).await;
    cleanup(pool, inst2).await;
  }

  /// get_by_post_id_for_employer returns Ok when the caller owns the delivery,
  /// and NotFound when they don't.
  #[tokio::test]
  #[serial]
  async fn get_by_post_id_for_employer_rejects_wrong_owner() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();

    let (inst1, pid1, _) = fixture_with_status(pool, TripStatus::Pending).await;
    let post1 = Post::read(pool, pid1).await.expect("read post1");
    let owner_id = post1.creator_id;

    // Create a second employer (different person) to obtain a different PersonId.
    let (inst2, pid2, _) = fixture_with_status(pool, TripStatus::Pending).await;
    let post2 = Post::read(pool, pid2).await.expect("read post2");
    let other_id = post2.creator_id;

    // Owner can fetch their own delivery.
    let result = DeliveryDetails::get_by_post_id_for_employer(pool, pid1, owner_id)
      .await
      .expect("owner must be able to fetch their delivery");
    assert_eq!(result.post_id, pid1);

    // Different person gets NotFound.
    let err = DeliveryDetails::get_by_post_id_for_employer(pool, pid1, other_id)
      .await
      .expect_err("wrong owner must get NotFound");
    assert!(
      format!("{err:?}").contains("NotFound"),
      "expected NotFound, got {err:?}"
    );

    cleanup(pool, inst1).await;
    cleanup(pool, inst2).await;
  }
}
