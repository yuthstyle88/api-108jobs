use crate::newtypes::WithdrawRequestId;
use crate::source::withdraw_request::{WithdrawRequest, WithdrawRequestInsertForm, WithdrawRequestUpdateForm};

#[cfg(feature = "full")]
use crate::{
    traits::Crud,
    utils::{get_conn, DbPool},
};

use diesel::ExpressionMethods;
#[cfg(feature = "full")]
use diesel::QueryDsl;
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;
#[cfg(feature = "full")]
use app_108jobs_db_schema_file::schema::withdraw_requests;
#[cfg(feature = "full")]
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

#[cfg(feature = "full")]
impl Crud for WithdrawRequest {
    type InsertForm = WithdrawRequestInsertForm;
    type UpdateForm = WithdrawRequestUpdateForm;
    type IdType = WithdrawRequestId;

    async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(withdraw_requests::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::DatabaseError)
    }

    async fn update(
        pool: &mut DbPool<'_>,
        id: Self::IdType,
        form: &Self::UpdateForm,
    ) -> FastJobResult<Self> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(withdraw_requests::table.find(id))
            .set(form)
            .get_result::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::DatabaseError)
    }
}

#[cfg(feature = "full")]
impl WithdrawRequest {
    /// Update a withdraw request by user_bank_account_id (or another field if needed)
    pub async fn update_by_user_bank_account_id(
        pool: &mut DbPool<'_>,
        user_bank_account_id: i32,
        form: &WithdrawRequestUpdateForm,
    ) -> FastJobResult<Self> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(
            withdraw_requests::table.filter(withdraw_requests::user_bank_account_id.eq(user_bank_account_id)),
        )
            .set(form)
            .get_result::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::DatabaseError)
    }

    /// Fetch all withdraw requests by a user
    pub async fn get_by_user(
        pool: &mut DbPool<'_>,
        user_id: i32,
    ) -> FastJobResult<Vec<Self>> {
        let conn = &mut get_conn(pool).await?;
        withdraw_requests::table
            .filter(withdraw_requests::local_user_id.eq(user_id))
            .order(withdraw_requests::created_at.desc())
            .load::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::DatabaseError)
    }
}
