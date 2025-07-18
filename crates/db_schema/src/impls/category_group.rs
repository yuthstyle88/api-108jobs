use crate::newtypes::CategoryGroupId;
use crate::source::category_group::{CategoryGroup, CategoryGroupInsertForm, CategoryGroupUpdateForm};
use crate::{
    traits::Crud,
    utils::{get_conn, DbPool},
};
use diesel::{insert_into, update, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{category_group};
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Crud for CategoryGroup {
    type InsertForm = CategoryGroupInsertForm;
    type UpdateForm = CategoryGroupUpdateForm;
    type IdType = CategoryGroupId;

    async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
        let conn = &mut get_conn(pool).await?;

        insert_into(category_group::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntCreateCategoryGroup)
    }

    async fn update(
        pool: &mut DbPool<'_>,
        id: Self::IdType,
        form: &Self::UpdateForm,
    ) -> FastJobResult<Self> {
        let conn = &mut get_conn(pool).await?;

        update(category_group::table.find(id))
            .set(form)
            .get_result::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntUpdateCategoryGroup)
    }
}
