use crate::{
    newtypes::PostId,
    source::delivery_location_current::{
        DeliveryLocationCurrent,
        DeliveryLocationCurrentInsertForm,
        DeliveryLocationCurrentUpdateForm,
    },
    traits::Crud,
    utils::{get_conn, DbPool},
};

use diesel::dsl::{insert_into, update};
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;

use app_108jobs_db_schema_file::schema::delivery_location_current;
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Crud for DeliveryLocationCurrent {
    type InsertForm = DeliveryLocationCurrentInsertForm;
    type UpdateForm = DeliveryLocationCurrentUpdateForm;
    type IdType = PostId;

    async fn create(
        pool: &mut DbPool<'_>,
        form: &Self::InsertForm,
    ) -> FastJobResult<Self> {
        let conn = &mut get_conn(pool).await?;

        insert_into(delivery_location_current::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntCreateDeliveryLocation)
    }

    async fn update(
        pool: &mut DbPool<'_>,
        post_id: PostId,
        form: &Self::UpdateForm,
    ) -> FastJobResult<Self> {
        let conn = &mut get_conn(pool).await?;

        update(delivery_location_current::table.find(post_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntUpdateDeliveryLocation)
    }
}
