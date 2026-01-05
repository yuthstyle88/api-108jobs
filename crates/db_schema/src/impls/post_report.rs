use crate::{
  newtypes::{PersonId, PostId, PostReportId},
  source::post_report::{PostReport, PostReportForm},
  traits::Reportable,
  utils::{get_conn, DbPool},
};
use chrono::Utc;
use diesel::{
  dsl::{insert_into, update},
  BoolExpressionMethods,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use app_108jobs_db_schema_file::schema::post_report;
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Reportable for PostReport {
  type Form = PostReportForm;
  type IdType = PostReportId;
  type ObjectIdType = PostId;

  async fn report(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_report::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateReport)
  }

  async fn resolve(
    pool: &mut DbPool<'_>,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(post_report::table.find(report_id))
      .set((
        post_report::resolved.eq(true),
        post_report::resolver_id.eq(by_resolver_id),
        post_report::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntResolveReport)
  }

  async fn resolve_apub(
    pool: &mut DbPool<'_>,
    object_id: Self::ObjectIdType,
    report_creator_id: PersonId,
    resolver_id: PersonId,
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(
      post_report::table.filter(
        post_report::post_id
          .eq(object_id)
          .and(post_report::creator_id.eq(report_creator_id)),
      ),
    )
    .set((
      post_report::resolved.eq(true),
      post_report::resolver_id.eq(resolver_id),
      post_report::updated_at.eq(Utc::now()),
    ))
    .execute(conn)
    .await
    .with_fastjob_type(FastJobErrorType::CouldntResolveReport)
  }

  async fn resolve_all_for_object(
    pool: &mut DbPool<'_>,
    post_id_: PostId,
    by_resolver_id: PersonId,
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(post_report::table.filter(post_report::post_id.eq(post_id_)))
      .set((
        post_report::resolved.eq(true),
        post_report::resolver_id.eq(by_resolver_id),
        post_report::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntResolveReport)
  }

  async fn unresolve(
    pool: &mut DbPool<'_>,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(post_report::table.find(report_id))
      .set((
        post_report::resolved.eq(false),
        post_report::resolver_id.eq(by_resolver_id),
        post_report::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntResolveReport)
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::{
    source::{
        category::{Category, CategoryInsertForm},
        instance::Instance,
        person::{Person, PersonInsertForm},
        post::{Post, PostInsertForm},
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use serial_test::serial;

  async fn init(pool: &mut DbPool<'_>) -> FastJobResult<(Person, PostReport)> {
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;
    let person_form = PersonInsertForm::test_form(inserted_instance.id, "jim");
    let person = Person::create(pool, &person_form).await?;

    let category_form = CategoryInsertForm::new(
      inserted_instance.id,
      "test category_4".to_string(),
      "nada".to_owned(),
    );
    let category = Category::create(pool, &category_form).await?;

    let form = PostInsertForm::new("A test post".into(), person.id, category.id);
    let post = Post::create(pool, &form).await?;

    let report_form = PostReportForm {
      post_id: post.id,
      creator_id: person.id,
      reason: "my reason".to_string(),
      ..Default::default()
    };
    let report = PostReport::report(pool, &report_form).await?;

    Ok((person, report))
  }

  #[tokio::test]
  #[serial]
  async fn test_resolve_post_report() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let (person, report) = init(pool).await?;

    let resolved_count = PostReport::resolve(pool, report.id, person.id).await?;
    assert_eq!(resolved_count, 1);

    let unresolved_count = PostReport::unresolve(pool, report.id, person.id).await?;
    assert_eq!(unresolved_count, 1);

    Person::delete(pool, person.id).await?;
    Post::delete(pool, report.post_id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_resolve_all_post_reports() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let (person, report) = init(pool).await?;

    let resolved_count =
      PostReport::resolve_all_for_object(pool, report.post_id, person.id).await?;
    assert_eq!(resolved_count, 1);

    Person::delete(pool, person.id).await?;
    Post::delete(pool, report.post_id).await?;

    Ok(())
  }
}
