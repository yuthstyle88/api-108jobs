use crate::{
  source::local_site::{LocalSite, LocalSiteInsertForm, LocalSiteUpdateForm},
  utils::{get_conn, DbPool},
};
use diesel::dsl::insert_into;
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::local_site;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl LocalSite {
  pub async fn create(pool: &mut DbPool<'_>, form: &LocalSiteInsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(local_site::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateSite)
  }

  pub async fn update(pool: &mut DbPool<'_>, form: &LocalSiteUpdateForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_site::table)
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateSite)
  }

  pub async fn delete(pool: &mut DbPool<'_>) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(local_site::table)
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::Deleted)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    source::{
      community::{Community, CommunityInsertForm, CommunityUpdateForm}
      ,
      person::{Person, PersonInsertForm}

      ,
    },
    test_data::TestData,
    traits::Crud,
    utils::{build_db_pool_for_tests, DbPool},
  };
  use lemmy_utils::error::FastJobResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  async fn read_local_site(pool: &mut DbPool<'_>) -> FastJobResult<LocalSite> {
    let conn = &mut get_conn(pool).await?;
    local_site::table
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  async fn prepare_site_with_community(
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<(TestData, Person, Community)> {
    let data = TestData::create(pool).await?;

    let new_person = PersonInsertForm::test_form(data.instance.id, "thommy_site_agg");
    let inserted_person = Person::create(pool, &new_person).await?;

    let new_community = CommunityInsertForm::new(
      data.instance.id,
      "TIL_site_agg".into(),
      "nada".to_owned(),
    );

    let inserted_community = Community::create(pool, &new_community).await?;

    Ok((data, inserted_person, inserted_community))
  }

  #[tokio::test]
  #[serial]
  async fn test_soft_delete() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let (data, inserted_person, inserted_community) = prepare_site_with_community(pool).await?;

    let site_aggregates_before = read_local_site(pool).await?;
    assert_eq!(1, site_aggregates_before.communities);

    Community::update(
      pool,
      inserted_community.id,
      &CommunityUpdateForm {
        deleted: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let site_aggregates_after_delete = read_local_site(pool).await?;
    assert_eq!(0, site_aggregates_after_delete.communities);

    Community::update(
      pool,
      inserted_community.id,
      &CommunityUpdateForm {
        deleted: Some(false),
        ..Default::default()
      },
    )
    .await?;

    Community::update(
      pool,
      inserted_community.id,
      &CommunityUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let site_aggregates_after_remove = read_local_site(pool).await?;
    assert_eq!(0, site_aggregates_after_remove.communities);

    Community::update(
      pool,
      inserted_community.id,
      &CommunityUpdateForm {
        deleted: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let site_aggregates_after_remove_delete = read_local_site(pool).await?;
    assert_eq!(0, site_aggregates_after_remove_delete.communities);

    Community::delete(pool, inserted_community.id).await?;
    Person::delete(pool, inserted_person.id).await?;
    data.delete(pool).await?;

    Ok(())
  }
}
