use crate::diesel::ExpressionMethods;
use crate::source::category::CategoryUpdateForm;
use crate::{
  newtypes::CategoryId,
  source::category::{Category, CategoryInsertForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use chrono::{DateTime, Utc};
use diesel::{dsl::insert_into, DecoratableTarget, QueryDsl};
use diesel_async::RunQueryDsl;
use diesel_ltree::Ltree;
use lemmy_db_schema_file::schema::category;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Crud for Category {
  type InsertForm = CategoryInsertForm;
  type UpdateForm = CategoryUpdateForm;
  type IdType = CategoryId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    Category::create(pool, None, form, None).await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(category::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateCategory)
  }
}

impl Category {
  pub async fn create(
    pool: &mut DbPool<'_>,
    timestamp: Option<DateTime<Utc>>,
    category_form: &CategoryInsertForm,
    parent_path: Option<&Ltree>,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    let category_form = (category_form, parent_path.map(|p| category::path.eq(p)));

    if let Some(ts) = timestamp {
      insert_into(category::table)
        .values(category_form)
        .on_conflict(category::slug)
        .filter_target(category::updated_at.lt(ts))
        .do_update()
        .set(category_form)
        .get_result::<Self>(conn)
        .await
        .with_fastjob_type(FastJobErrorType::CouldntCreateCategory)
    } else {
      insert_into(category::table)
        .values(category_form)
        .get_result::<Self>(conn)
        .await
        .with_fastjob_type(FastJobErrorType::CouldntCreateCategory)
    }
  }
}
