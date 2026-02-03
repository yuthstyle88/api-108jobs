use crate::{
    newtypes::{CommentId, DeliveryDetailsId, PersonId, PostId, RiderId},
    source::delivery_details::{
        DeliveryDetails,
        DeliveryDetailsInsertForm,
        DeliveryDetailsUpdateForm,
    },
    traits::Crud,
    utils::{get_conn, DbPool},
};

use diesel::dsl::{insert_into, update};
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use chrono::{DateTime, Utc};

use app_108jobs_db_schema_file::schema::delivery_details;
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use app_108jobs_db_schema_file::schema::{post as post_tbl, rider as rider_tbl};
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
