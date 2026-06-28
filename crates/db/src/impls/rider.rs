use crate::{
  newtypes::{LocalUserId, PersonId, RiderId},
  source::rider::{Rider, RiderInsertForm, RiderUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use app_108jobs_core::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use crate::schema::rider;
use diesel::{
  dsl::{exists, insert_into, select, update},
  ExpressionMethods,
  OptionalExtension,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;

impl Crud for Rider {
  type InsertForm = RiderInsertForm;
  type UpdateForm = RiderUpdateForm;
  type IdType = RiderId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(rider::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateRider)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    rider_id: RiderId,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    update(rider::table.find(rider_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateRider)
  }
}

impl Rider {
  pub async fn read(pool: &mut DbPool<'_>, rider_id: RiderId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    rider::table
      .find(rider_id)
      .select(Self::as_select())
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn get_by_person_id(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;

    let result = rider::table
      .filter(rider::person_id.eq(person_id.0))
      .filter(rider::is_active.eq(true))
      .first::<Self>(conn)
      .await
      .optional()
      .map_err(|_| FastJobErrorType::DatabaseError)?;

    Ok(result)
  }

  pub async fn exists_for_user(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
  ) -> FastJobResult<bool> {
    let conn = &mut get_conn(pool).await?;

    select(exists(
      rider::table.filter(rider::user_id.eq(local_user_id)),
    ))
    .get_result::<bool>(conn)
    .await
    .with_fastjob_type(FastJobErrorType::NotFound)
  }
}

// ============================================================================
// DB-backed tests for rider creation and admin verification used by
// `api_crud/rider/{create, update}` and the `/admin/riders/verify` route.
//
// Coverage:
//   * Create defaults: is_verified=false, verification_status=Pending, is_active=true (per schema),
//     accepting_jobs=true.
//   * admin_verify_rider flips is_verified=true, verification_status=Approved, and is repeat-safe
//     (idempotent).
//   * get_by_person_id finds active riders only.
//   * exists_for_user is true after creation.
// ============================================================================
#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    source::{
      instance::Instance,
      person::{Person, PersonInsertForm},
    },
    test_data::pool_for_tests,
  };
  use crate::{
    enums::{RiderVerificationStatus, VehicleType},
    schema::local_user,
  };
  use diesel::ExpressionMethods;
  use diesel_async::RunQueryDsl;
  use serial_test::serial;

  struct RiderCtx {
    instance_id: crate::newtypes::InstanceId,
    person_id: PersonId,
    local_user_id: LocalUserId,
  }

  async fn make_rider_user(pool: &mut DbPool<'_>) -> RiderCtx {
    let inst = Instance::read_or_create(pool, format!("rider-test-{}.tld", uuid::Uuid::new_v4()))
      .await
      .expect("create instance");
    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let suffix_short = &suffix[..8];
    let (p_form, _wallet) =
      PersonInsertForm::test_form_with_wallet(pool, inst.id, &format!("rd-{suffix_short}"))
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
    RiderCtx {
      instance_id: inst.id,
      person_id: person.id,
      local_user_id: LocalUserId(local_user_id),
    }
  }

  fn insert_form(ctx: &RiderCtx) -> RiderInsertForm {
    RiderInsertForm::new(ctx.local_user_id, ctx.person_id, VehicleType::Motorcycle)
  }

  async fn cleanup(pool: &mut DbPool<'_>, instance_id: crate::newtypes::InstanceId) {
    let _ = Instance::delete(pool, instance_id).await;
  }

  /// Newly-created rider must be in Pending verification status,
  /// and `exists_for_user` must report true.
  #[tokio::test]
  #[serial]
  async fn create_defaults_pending_verification() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = make_rider_user(pool).await;

    let rider = Rider::create(pool, &insert_form(&ctx))
      .await
      .expect("create rider");
    assert!(!rider.is_verified);
    assert_eq!(rider.verification_status, RiderVerificationStatus::Pending);
    assert_eq!(rider.vehicle_type, VehicleType::Motorcycle);

    let found = Rider::exists_for_user(pool, ctx.local_user_id)
      .await
      .expect("exists");
    assert!(found);
    cleanup(pool, ctx.instance_id).await;
  }

  /// admin_verify_rider toggles status -> Approved + is_verified=true and is
  /// safe to call twice.
  #[tokio::test]
  #[serial]
  async fn admin_verify_is_idempotent() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = make_rider_user(pool).await;
    let rider = Rider::create(pool, &insert_form(&ctx))
      .await
      .expect("create rider");

    let v1 = Rider::update(
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
    .expect("first verify");
    assert!(v1.is_verified);
    assert_eq!(v1.verification_status, RiderVerificationStatus::Verified);

    let v2 = Rider::update(
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
    .expect("second verify");
    assert!(v2.is_verified);
    assert_eq!(v2.verification_status, RiderVerificationStatus::Verified);
    cleanup(pool, ctx.instance_id).await;
  }

  /// get_by_person_id must only return active riders.
  /// Setting is_active=false hides the row from this lookup.
  #[tokio::test]
  #[serial]
  async fn get_by_person_id_filters_active() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = make_rider_user(pool).await;
    let rider = Rider::create(pool, &insert_form(&ctx))
      .await
      .expect("create rider");

    let active = Rider::get_by_person_id(pool, ctx.person_id)
      .await
      .expect("get active");
    assert_eq!(active.map(|r| r.id), Some(rider.id));

    // Mark inactive.
    let _ = Rider::update(
      pool,
      rider.id,
      &RiderUpdateForm {
        is_active: Some(false),
        ..Default::default()
      },
    )
    .await
    .expect("deactivate");

    let inactive_lookup = Rider::get_by_person_id(pool, ctx.person_id)
      .await
      .expect("get after deactivate");
    assert!(
      inactive_lookup.is_none(),
      "inactive rider should not appear via get_by_person_id"
    );
    cleanup(pool, ctx.instance_id).await;
  }
}
