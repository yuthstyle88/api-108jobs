use crate::newtypes::{LocalUserId, UserKeyMappingId};

#[cfg(feature = "full")]
use crate::{
    source::user_key_mapping::{UserKeyMapping, UserKeyMappingInsertForm, UserKeyMappingUpdateForm},
    traits::Crud,
    utils::{get_conn, DbPool},
};
#[cfg(feature = "full")]
use diesel::QueryDsl;
use diesel::ExpressionMethods;
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;

#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::user_key_mappings;
use lemmy_db_schema_file::schema::user_key_mappings::dsl as ukm;

#[cfg(feature = "full")]
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

#[cfg(feature = "full")]
impl Crud for UserKeyMapping {
    type InsertForm = UserKeyMappingInsertForm;
    type UpdateForm = UserKeyMappingUpdateForm;
    type IdType = UserKeyMappingId;

    async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(user_key_mappings::table)
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
        diesel::update(user_key_mappings::table.find(id))
            .set(form)
            .get_result::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::DatabaseError)
    }
}

#[cfg(feature = "full")]
impl UserKeyMapping {
    pub async fn upsert(
        pool: &mut DbPool<'_>,
        local_user_id: LocalUserId,
        hashed_password: String,
        real_key: String,
    ) -> FastJobResult<Self> {
        let conn = &mut get_conn(pool).await?;

        let now = chrono::Utc::now();

        diesel::insert_into(user_key_mappings::table)
            .values(UserKeyMappingInsertForm {
                local_user_id,
                hashed_password: hashed_password.clone(),
                real_key: real_key.clone(),
                created_at: Some(now),
                updated_at: None,
            })
            .on_conflict(ukm::local_user_id)
            .do_update()
            .set((
                ukm::hashed_password.eq(hashed_password),
                ukm::real_key.eq(real_key),
                ukm::updated_at.eq(now),
            ))
            .get_result::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::DatabaseError)
    }
}
