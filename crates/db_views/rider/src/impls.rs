use crate::RiderView;
use app_108jobs_db_schema::{
  newtypes::{DecodedCursor, LocalUserId, PaginationCursor, RiderId},
  source::rider::{rider_keys as key, Rider},
  traits::PaginationCursorBuilder,
  utils::{get_conn, paginate, Commented, DbPool},
};
use app_108jobs_db_schema_file::schema::{person, rider};
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use diesel::{
  self,
  query_builder::AsQuery,
  ExpressionMethods,
  JoinOnDsl,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;

impl PaginationCursorBuilder for RiderView {
  type CursorData = Rider;

  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::v2_i32(self.rider.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    let decoded = cursor.decode()?;

    let id = match decoded {
      DecodedCursor::I32(id) => id,
      DecodedCursor::I64(id) => id as i32,
      DecodedCursor::Composite(parts) => parts[0].1,
    };

    Rider::read(pool, RiderId(id)).await
  }
}

impl RiderView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    rider::table.inner_join(person::table.on(rider::person_id.eq(person::id)))
  }

  /// Read a single rider
  pub async fn read(pool: &mut DbPool<'_>, rider_id: RiderId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    let query = Self::joins()
      .filter(rider::id.eq(rider_id))
      .select(Self::as_select());

    Commented::new(query)
      .text("RiderView::read")
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  /// Read a rider by the owning local user id
  pub async fn read_by_user_id(pool: &mut DbPool<'_>, user_id: LocalUserId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    let query = Self::joins()
      .filter(rider::user_id.eq(user_id))
      .select(Self::as_select());

    Commented::new(query)
      .text("RiderView::read_by_user_id")
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn list(
    pool: &mut DbPool<'_>,
    cursor_data: Option<Rider>,
    page_back: Option<bool>,
    limit: Option<i64>,
    verified: Option<bool>,
  ) -> FastJobResult<Vec<RiderView>> {
    use app_108jobs_db_schema_file::schema::rider;

    let conn = &mut get_conn(pool).await?;

    let limit = limit.unwrap_or(20);

    let mut query = Self::joins()
      .select(Self::as_select())
      .limit(limit)
      .into_boxed();

    // is_verified filter
    let is_verified = verified.unwrap_or(false);

    query = query.filter(rider::is_verified.eq(is_verified));

    // Active riders only
    query = query.filter(rider::is_active.eq(true));

    let paginated = paginate(query, SortDirection::Desc, cursor_data, None, page_back)
      .then_order_by(key::joined_at)
      .then_order_by(key::id);

    let query = paginated.as_query();

    Commented::new(query)
      .text("RiderView::list")
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}

// ============================================================================
// Smoke tests: happy-path integration tests for the RiderView query layer.
//
// Coverage:
//   * Full rider status lifecycle: create → admin-verify → online → accepting toggle.
//   * RiderView::read and read_by_user_id return the correct joined row.
//   * RiderView::list filters by is_verified; unverified riders are excluded.
//
// Run with: cargo test --features full -p app_108jobs_db_views_rider
// ============================================================================
#[cfg(test)]
mod tests {
  use crate::RiderView;
  use app_108jobs_db_schema::{
    newtypes::{InstanceId, LocalUserId, PersonId},
    source::{
      instance::Instance,
      person::{Person, PersonInsertForm},
      rider::{Rider, RiderInsertForm, RiderUpdateForm},
    },
    test_data::pool_for_tests,
    traits::Crud,
    utils::{get_conn, DbPool},
  };
  use app_108jobs_db_schema_file::{
    enums::{RiderVerificationStatus, VehicleType},
    schema::local_user,
  };
  use diesel::ExpressionMethods;
  use diesel_async::RunQueryDsl;
  use serial_test::serial;

  struct RiderFixture {
    instance_id: InstanceId,
    person_id: PersonId,
    local_user_id: LocalUserId,
  }

  async fn make_rider_fixture(pool: &mut DbPool<'_>) -> RiderFixture {
    let inst =
      Instance::read_or_create(pool, format!("rv-smoke-{}.tld", uuid::Uuid::new_v4().simple()))
        .await
        .expect("create instance");
    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let (p_form, _wallet) =
      PersonInsertForm::test_form_with_wallet(pool, inst.id, &format!("rv-{}", &suffix[..8]))
        .await
        .expect("test_form_with_wallet");
    let person = Person::create(pool, &p_form).await.expect("create person");

    let conn = &mut get_conn(pool).await.expect("get conn");
    let local_user_id: i32 = diesel::insert_into(local_user::table)
      .values((
        local_user::person_id.eq(person.id),
        local_user::password_encrypted.eq::<Option<String>>(None),
      ))
      .returning(local_user::id)
      .get_result(conn)
      .await
      .expect("insert local_user");

    RiderFixture {
      instance_id: inst.id,
      person_id: person.id,
      local_user_id: LocalUserId(local_user_id),
    }
  }

  fn new_rider_form(fix: &RiderFixture) -> RiderInsertForm {
    RiderInsertForm::new(fix.local_user_id, fix.person_id, VehicleType::Motorcycle)
  }

  async fn cleanup(pool: &mut DbPool<'_>, instance_id: InstanceId) {
    let _ = Instance::delete(pool, instance_id).await;
  }

  /// Full rider status lifecycle: create → admin-verify → set online → disable accepting.
  #[tokio::test]
  #[serial]
  async fn smoke_rider_lifecycle() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let fix = make_rider_fixture(pool).await;

    let rider = Rider::create(pool, &new_rider_form(&fix))
      .await
      .expect("create rider");
    assert_eq!(rider.verification_status, RiderVerificationStatus::Pending);
    assert!(!rider.is_verified);

    let rider = Rider::update(
      pool,
      rider.id,
      &RiderUpdateForm {
        is_verified: Some(true),
        verification_status: Some(RiderVerificationStatus::Verified),
        verified_at: Some(Some(chrono::Utc::now())),
        ..Default::default()
      },
    )
    .await
    .expect("verify rider");
    assert!(rider.is_verified);
    assert_eq!(rider.verification_status, RiderVerificationStatus::Verified);

    let rider = Rider::update(
      pool,
      rider.id,
      &RiderUpdateForm {
        is_online: Some(true),
        ..Default::default()
      },
    )
    .await
    .expect("set online");
    assert!(rider.is_online);

    let rider = Rider::update(
      pool,
      rider.id,
      &RiderUpdateForm {
        accepting_jobs: Some(false),
        ..Default::default()
      },
    )
    .await
    .expect("disable accepting");
    assert!(!rider.accepting_jobs);

    cleanup(pool, fix.instance_id).await;
  }

  /// RiderView::read and read_by_user_id return the correct joined person+rider row.
  #[tokio::test]
  #[serial]
  async fn smoke_rider_view_read() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let fix = make_rider_fixture(pool).await;

    let rider = Rider::create(pool, &new_rider_form(&fix))
      .await
      .expect("create rider");

    let by_id = RiderView::read(pool, rider.id)
      .await
      .expect("RiderView::read");
    assert_eq!(by_id.rider.id, rider.id);
    assert_eq!(by_id.person.id, fix.person_id);

    let by_user = RiderView::read_by_user_id(pool, fix.local_user_id)
      .await
      .expect("RiderView::read_by_user_id");
    assert_eq!(by_user.rider.id, rider.id);

    cleanup(pool, fix.instance_id).await;
  }

  /// RiderView::list filters by verification: unverified riders must not appear
  /// in the verified list, and verified riders must.
  #[tokio::test]
  #[serial]
  async fn smoke_rider_view_list_filters_by_verified() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();

    let fix_a = make_rider_fixture(pool).await;
    let rider_a = Rider::create(pool, &new_rider_form(&fix_a))
      .await
      .expect("create rider_a");
    // rider_a stays Pending (unverified)

    let fix_b = make_rider_fixture(pool).await;
    let rider_b = Rider::create(pool, &new_rider_form(&fix_b))
      .await
      .expect("create rider_b");
    Rider::update(
      pool,
      rider_b.id,
      &RiderUpdateForm {
        is_verified: Some(true),
        verification_status: Some(RiderVerificationStatus::Verified),
        verified_at: Some(Some(chrono::Utc::now())),
        ..Default::default()
      },
    )
    .await
    .expect("verify rider_b");

    let verified_list = RiderView::list(pool, None, None, Some(100), Some(true))
      .await
      .expect("list verified");
    let ids: Vec<_> = verified_list.iter().map(|v| v.rider.id).collect();
    assert!(
      ids.contains(&rider_b.id),
      "verified rider must appear in the verified list"
    );
    assert!(
      !ids.contains(&rider_a.id),
      "unverified rider must not appear in the verified list"
    );

    cleanup(pool, fix_a.instance_id).await;
    cleanup(pool, fix_b.instance_id).await;
  }
}
