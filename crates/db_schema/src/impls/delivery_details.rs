use crate::{
    newtypes::{CoinId, CommentId, DeliveryDetailsId, LocalUserId, PersonId, PostId, RiderId, WalletId},
    source::{
        delivery_details::{
            DeliveryDetails,
            DeliveryDetailsInsertForm,
            DeliveryDetailsUpdateForm,
        },
        local_user::LocalUser,
        post::Post,
        rider::Rider,
        wallet::{TxKind, WalletModel, WalletTransactionInsertForm},
    },
    traits::Crud,
    utils::{get_conn, DbPool},
};

use diesel::dsl::{insert_into, update};
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use app_108jobs_db_schema_file::schema::delivery_details;
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use app_108jobs_db_schema_file::schema::{local_user as local_user_tbl, post as post_tbl, rider as rider_tbl};
use app_108jobs_db_schema_file::enums::{DeliveryStatus, PostKind, RiderVerificationStatus};

impl Crud for DeliveryDetails {
    type InsertForm = DeliveryDetailsInsertForm;
    type UpdateForm = DeliveryDetailsUpdateForm;
    type IdType = DeliveryDetailsId;

    async fn create(
        pool: &mut DbPool<'_>,
        form: &Self::InsertForm,
    ) -> FastJobResult<Self> {
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
        use diesel::dsl::select;
        use diesel::ExpressionMethods;

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
                .filter(delivery_details::dsl::status.ne(DeliveryStatus::Cancelled))
                .filter(delivery_details::dsl::status.ne(DeliveryStatus::Delivered)),
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
    pub fn can_transition_to(&self, new_status: DeliveryStatus) -> bool {
        use DeliveryStatus::*;

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
        new_status: DeliveryStatus,
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
            return Err(FastJobErrorType::InvalidField(format!(
                "Cannot transition from {:?} to {:?}",
                current_delivery.status, new_status
            ))
            .into());
        }

        // Determine the cancellation reason to save
        // If cancelling, use the provided reason or clear it if not cancelling
        let reason_to_save = match new_status {
            DeliveryStatus::Cancelled => cancellation_reason,
            _ => None, // Clear reason for non-cancelled statuses
        };

        // Perform the update
        let updated_delivery = update(delivery_details::dsl::delivery_details
            .filter(delivery_details::dsl::post_id.eq(post_id.0)))
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
    pub async fn get_by_post_id(
        pool: &mut DbPool<'_>,
        post_id: PostId,
    ) -> FastJobResult<Self> {
        let conn = &mut get_conn(pool).await?;

        delivery_details::dsl::delivery_details
            .filter(delivery_details::dsl::post_id.eq(post_id))
            .first::<Self>(conn)
            .await
            .map_err(|_| FastJobErrorType::NotFound.into())
    }

    /// Get all active deliveries (in progress: Assigned, EnRouteToPickup, PickedUp, EnRouteToDropoff).
    pub async fn get_all_active(
        pool: &mut DbPool<'_>,
    ) -> FastJobResult<Vec<Self>> {
        use diesel::QueryDsl;
        let conn = &mut get_conn(pool).await?;

        let results = delivery_details::dsl::delivery_details
            .filter(delivery_details::dsl::status.eq_any(vec![
                DeliveryStatus::Assigned,
                DeliveryStatus::EnRouteToPickup,
                DeliveryStatus::PickedUp,
                DeliveryStatus::EnRouteToDropoff,
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
        if current_delivery.status != DeliveryStatus::Pending {
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
        let updated_delivery = update(delivery_details::dsl::delivery_details
            .filter(delivery_details::dsl::post_id.eq(post_id.0)))
            .set((
                delivery_details::dsl::assigned_rider_id.eq(rider_id.0),
                delivery_details::dsl::assigned_at.eq(Utc::now()),
                delivery_details::dsl::assigned_by_person_id.eq(assigned_by_person_id.0),
                delivery_details::dsl::linked_comment_id.eq(comment_id.0),
                delivery_details::dsl::sender_name.eq(sender_name),
                delivery_details::dsl::sender_phone.eq(sender_phone),
                delivery_details::dsl::receiver_name.eq(receiver_name),
                delivery_details::dsl::receiver_phone.eq(receiver_phone),
                delivery_details::dsl::status.eq(DeliveryStatus::Assigned),
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

        conn.run_transaction(|conn| {
            async move {
                // First, fetch the current delivery details to validate
                let current_delivery = delivery_details::dsl::delivery_details
                    .filter(delivery_details::dsl::post_id.eq(post_id.0))
                    .first::<Self>(conn)
                    .await
                    .map_err(|_| FastJobErrorType::NotFound)?;

                // Can only assign from Pending status
                if current_delivery.status != DeliveryStatus::Pending {
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
                    idempotency_key: Uuid::new_v4().to_string(),
                };
                let _ = WalletModel::hold(&mut pool, &tx_form).await?;

                // Perform the assignment with sender/receiver info and escrow details
                let conn = &mut get_conn(&mut pool).await?;
                let updated_delivery = update(delivery_details::dsl::delivery_details
                    .filter(delivery_details::dsl::post_id.eq(post_id.0)))
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
                        delivery_details::dsl::status.eq(DeliveryStatus::Assigned),
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
        if current_delivery.status != DeliveryStatus::Assigned {
            return Err(FastJobErrorType::CannotUnassignFromStatus.into());
        }

        // Perform the unassignment
        let updated_delivery = update(delivery_details::dsl::delivery_details
            .filter(delivery_details::dsl::post_id.eq(post_id.0)))
            .set((
                delivery_details::dsl::assigned_rider_id.eq(Option::<i32>::None),
                delivery_details::dsl::assigned_at.eq(Option::<DateTime<Utc>>::None),
                delivery_details::dsl::assigned_by_person_id.eq(Option::<i32>::None),
                delivery_details::dsl::linked_comment_id.eq(Option::<i32>::None),
                delivery_details::dsl::status.eq(DeliveryStatus::Pending),
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

        conn.run_transaction(|conn| {
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
                if current_delivery.status != DeliveryStatus::Delivered {
                    return Err(FastJobErrorType::CannotConfirmNonDeliveredDelivery.into());
                }

                // Check if already confirmed
                if current_delivery.employer_confirmed_at.is_some() {
                    // Already confirmed - idempotent, return current state
                    return Ok(current_delivery);
                }

                // Get the assigned rider
                let rider_id = current_delivery.assigned_rider_id
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
                    idempotency_key: Uuid::new_v4().to_string(),
                };
                WalletModel::deposit_from_platform(&mut pool, &tx_form, coin_id, platform_wallet_id).await?;

                // Update the delivery with confirmation timestamp
                let conn = &mut get_conn(&mut pool).await?;
                let updated_delivery = update(delivery_details::dsl::delivery_details
                    .filter(delivery_details::dsl::post_id.eq(post_id.0)))
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
                DeliveryStatus::Assigned,
                DeliveryStatus::EnRouteToPickup,
                DeliveryStatus::PickedUp,
                DeliveryStatus::EnRouteToDropoff,
            ]))
            .first::<Self>(conn)
            .await
            .ok();

        Ok(result)
    }

    /// Get all completed deliveries (status = Delivered).
    pub async fn get_all_completed(
        pool: &mut DbPool<'_>,
    ) -> FastJobResult<Vec<Self>> {
        use diesel::QueryDsl;
        let conn = &mut get_conn(pool).await?;

        let results = delivery_details::dsl::delivery_details
            .filter(delivery_details::dsl::status.eq(DeliveryStatus::Delivered))
            .order(delivery_details::dsl::created_at.desc())
            .load::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntUpdateDeliveryDetails)?;

        Ok(results)
    }

    /// Get all cancelled deliveries (status = Cancelled).
    pub async fn get_all_cancelled(
        pool: &mut DbPool<'_>,
    ) -> FastJobResult<Vec<Self>> {
        use diesel::QueryDsl;
        let conn = &mut get_conn(pool).await?;

        let results = delivery_details::dsl::delivery_details
            .filter(delivery_details::dsl::status.eq(DeliveryStatus::Cancelled))
            .order(delivery_details::dsl::created_at.desc())
            .load::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntUpdateDeliveryDetails)?;

        Ok(results)
    }
}
