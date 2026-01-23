use crate::{
    newtypes::{DeliveryDetailsId, PersonId, PostId, RiderId},
    source::delivery_details::{
        DeliveryDetails,
        DeliveryDetailsInsertForm,
        DeliveryDetailsUpdateForm,
    },
    traits::Crud,
    utils::{get_conn, DbPool},
};

use diesel::dsl::{insert_into, update};
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;

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
            return Err(FastJobErrorType::InvalidField("invalid delivery post".to_string()).into());
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
            return Err(FastJobErrorType::InvalidField("delivery is not active".to_string()).into());
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
            Err(FastJobErrorType::InvalidField("not a rider".to_string()).into())
        }
    }
}
