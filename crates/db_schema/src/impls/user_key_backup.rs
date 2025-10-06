use crate::newtypes::{LocalUserId, UserKeyBackupId};

#[cfg(feature = "full")]
use crate::{
    source::user_key_backup::{
        UserKeyBackup,
        UserKeyBackupInsertForm,
        UserKeyBackupUpdateForm,
    },
    traits::Crud,
    utils::{get_conn, DbPool},
};
#[cfg(feature = "full")]
use diesel::QueryDsl;
use diesel::ExpressionMethods;
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;

#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::user_key_backups;
use lemmy_db_schema_file::schema::user_key_backups::dsl as ukb;

#[cfg(feature = "full")]
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

#[cfg(feature = "full")]
impl Crud for UserKeyBackup {
    type InsertForm = UserKeyBackupInsertForm;
    type UpdateForm = UserKeyBackupUpdateForm;
    type IdType = UserKeyBackupId;

    async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(user_key_backups::table)
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
        diesel::update(user_key_backups::table.find(id))
            .set(form)
            .get_result::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::DatabaseError)
    }
}

#[cfg(feature = "full")]
impl UserKeyBackup {
    pub async fn upsert(
        pool: &mut DbPool<'_>,
        local_user_id: LocalUserId,
        encrypted_private_key: Vec<u8>,
        iv: Vec<u8>,
        salt: Vec<u8>,
    ) -> FastJobResult<Self> {
        let conn = &mut get_conn(pool).await?;
        let now = chrono::Utc::now();

        diesel::insert_into(user_key_backups::table)
            .values(UserKeyBackupInsertForm {
                local_user_id,
                encrypted_private_key: encrypted_private_key.clone(),
                iv: iv.clone(),
                salt: salt.clone(),
                created_at: Some(now),
                updated_at: None,
            })
            .on_conflict(ukb::local_user_id)
            .do_update()
            .set((
                ukb::encrypted_private_key.eq(encrypted_private_key),
                ukb::iv.eq(iv),
                ukb::salt.eq(salt),
                ukb::updated_at.eq(now),
            ))
            .get_result::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::DatabaseError)
    }
}
