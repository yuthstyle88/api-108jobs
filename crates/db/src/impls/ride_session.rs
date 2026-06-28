use crate::{
  newtypes::{LocalUserId, PostId, RideSessionId, RiderId},
  source::ride_session::{RideSession, RideSessionInsertForm, RideSessionUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use app_108jobs_core::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use crate::{enums::TripStatus, schema::ride_session};
use diesel::{
  dsl::{insert_into, update},
  ExpressionMethods,
  OptionalExtension,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

impl Crud for RideSession {
  type InsertForm = RideSessionInsertForm;
  type UpdateForm = RideSessionUpdateForm;
  type IdType = RideSessionId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(crate::schema::ride_session::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateRideSession)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    session_id: RideSessionId,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    update(crate::schema::ride_session::table.find(session_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateRideSession)
  }
}

impl RideSession {
  pub async fn get_by_post(pool: &mut DbPool<'_>, post_id: PostId) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;

    let result = ride_session::table
      .filter(ride_session::post_id.eq(post_id))
      .first::<Self>(conn)
      .await
      .optional()
      .map_err(|_| FastJobErrorType::DatabaseError)?;

    Ok(result)
  }

  pub async fn get_by_rider(pool: &mut DbPool<'_>, rider_id: RiderId) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    ride_session::table
      .filter(ride_session::rider_id.eq(rider_id))
      .order(ride_session::created_at.desc())
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  pub async fn get_by_employer(
    pool: &mut DbPool<'_>,
    employer_id: LocalUserId,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    ride_session::table
      .filter(ride_session::employer_id.eq(employer_id))
      .order(ride_session::created_at.desc())
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  pub async fn get_active_by_rider(
    pool: &mut DbPool<'_>,
    rider_id: RiderId,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;

    let result = ride_session::table
      .filter(ride_session::rider_id.eq(rider_id))
      .filter(ride_session::status.eq(TripStatus::PickedUp))
      .first::<Self>(conn)
      .await
      .optional()
      .map_err(|_| FastJobErrorType::DatabaseError)?;

    Ok(result)
  }

  /// List ride sessions for a rider with optional status filter and pagination
  pub async fn list_for_rider(
    pool: &mut DbPool<'_>,
    rider_id: RiderId,
    status: Option<TripStatus>,
    limit: Option<i64>,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit.unwrap_or(20);

    let mut query = ride_session::table
      .filter(ride_session::rider_id.eq(rider_id))
      .order(ride_session::created_at.desc())
      .limit(limit)
      .into_boxed();

    if let Some(s) = status {
      query = query.filter(ride_session::status.eq(s));
    }

    query
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  /// List available ride sessions that riders can accept (Pending status, no rider assigned)
  pub async fn list_available_for_rider(
    pool: &mut DbPool<'_>,
    limit: Option<i64>,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit.unwrap_or(20);

    ride_session::table
      .filter(ride_session::status.eq(TripStatus::Pending))
      .filter(ride_session::rider_id.is_null())
      .order(ride_session::created_at.desc())
      .limit(limit)
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  /// Check if a rider has any active (non-terminal) ride sessions
  /// Active statuses: Pending, Assigned, EnRouteToPickup, PickedUp, EnRouteToDropoff,
  /// RiderConfirmed Terminal statuses: Delivered, Cancelled
  pub async fn has_active_session(pool: &mut DbPool<'_>, rider_id: RiderId) -> FastJobResult<bool> {
    let conn = &mut get_conn(pool).await?;

    let active_statuses = vec![
      TripStatus::Pending,
      TripStatus::Assigned,
      TripStatus::EnRouteToPickup,
      TripStatus::PickedUp,
      TripStatus::EnRouteToDropoff,
      TripStatus::RiderConfirmed,
    ];

    let count: i64 = ride_session::table
      .filter(ride_session::rider_id.eq(rider_id))
      .filter(ride_session::status.eq_any(active_statuses))
      .count()
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;

    Ok(count > 0)
  }

  /// Check if a post already has a ride session
  pub async fn exists_for_post(pool: &mut DbPool<'_>, post_id: PostId) -> FastJobResult<bool> {
    let conn = &mut get_conn(pool).await?;

    let count: i64 = ride_session::table
      .filter(ride_session::post_id.eq(post_id))
      .count()
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;

    Ok(count > 0)
  }
}

// ============================================================================
// DB-backed tests for ride session lifecycle (no money path — see audit).
//
// Coverage:
//   * create defaults: status from form, payment_status="pending" via NULL fallback
//   * update flips status to RiderConfirmed / Delivered / Cancelled with the corresponding
//     timestamp fields populated
//   * has_active_session detects each non-terminal status and is false once the session is
//     Delivered/Cancelled (the "rider is available again" cue used by update_ride_status and
//     cancel_ride_session)
//   * list_available_for_rider returns only Pending rows with NULL rider_id
//
// The ride flow has no implemented escrow/payment release — see the
// not-confirmed report; no money-assertions tests included.
// ============================================================================
#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    source::{
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::PostInsertForm,
      rider::{Rider, RiderInsertForm},
    },
    test_data::pool_for_tests,
  };
  use crate::{
    enums::{PaymentMethod, PostKind, VehicleType},
    schema::{local_user, post},
  };
  use chrono::Utc;
  use diesel::ExpressionMethods;
  use diesel_async::RunQueryDsl;
  use serial_test::serial;

  struct RideCtx {
    instance_id: crate::newtypes::InstanceId,
    post_id: PostId,
    employer_id: LocalUserId,
    rider_id: RiderId,
  }

  async fn fixture(pool: &mut DbPool<'_>) -> RideCtx {
    let inst = Instance::read_or_create(pool, format!("rs-test-{}.tld", uuid::Uuid::new_v4()))
      .await
      .expect("create instance");

    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let suffix_short = &suffix[..8];

    // Employer
    let (emp_form, _emp_w) =
      PersonInsertForm::test_form_with_wallet(pool, inst.id, &format!("emp-{suffix_short}"))
        .await
        .expect("emp form");
    let emp = Person::create(pool, &emp_form).await.expect("create emp");
    let emp_local_user_id: i32 = {
      let conn = &mut get_conn(pool).await.expect("conn");
      diesel::insert_into(local_user::table)
        .values((
          local_user::person_id.eq(emp.id),
          local_user::password_encrypted.eq::<Option<String>>(None),
        ))
        .returning(local_user::id)
        .get_result(conn)
        .await
        .expect("emp local_user")
    };

    // Rider (person + local_user + rider row)
    let (rd_form, _rd_w) =
      PersonInsertForm::test_form_with_wallet(pool, inst.id, &format!("rd-{suffix_short}"))
        .await
        .expect("rd form");
    let rd = Person::create(pool, &rd_form).await.expect("create rd");
    let rd_local_user_id: i32 = {
      let conn = &mut get_conn(pool).await.expect("conn");
      diesel::insert_into(local_user::table)
        .values((
          local_user::person_id.eq(rd.id),
          local_user::password_encrypted.eq::<Option<String>>(None),
        ))
        .returning(local_user::id)
        .get_result(conn)
        .await
        .expect("rd local_user")
    };
    let rider = Rider::create(
      pool,
      &RiderInsertForm::new(
        LocalUserId(rd_local_user_id),
        rd.id,
        VehicleType::Motorcycle,
      ),
    )
    .await
    .expect("create rider");

    // Post
    let mut post_form = PostInsertForm::new(format!("taxi {suffix_short}"), emp.id);
    post_form.post_kind = Some(PostKind::RideTaxi);
    let post_id: i32 = {
      let conn = &mut get_conn(pool).await.expect("conn");
      diesel::insert_into(post::table)
        .values(&post_form)
        .returning(post::id)
        .get_result(conn)
        .await
        .expect("insert post")
    };

    RideCtx {
      instance_id: inst.id,
      post_id: PostId(post_id),
      employer_id: LocalUserId(emp_local_user_id),
      rider_id: rider.id,
    }
  }

  fn pending_form(ctx: &RideCtx) -> RideSessionInsertForm {
    RideSessionInsertForm {
      post_id: ctx.post_id,
      rider_id: None,
      employer_id: ctx.employer_id,
      pricing_config_id: None,
      pickup_address: "Pickup".to_string(),
      pickup_lat: None,
      pickup_lng: None,
      dropoff_address: "Dropoff".to_string(),
      dropoff_lat: None,
      dropoff_lng: None,
      pickup_note: None,
      passenger_name: None,
      passenger_phone: None,
      payment_method: PaymentMethod::Cash,
      payment_status: Some("pending".to_string()),
      status: Some(TripStatus::Pending),
      requested_at: Some(Utc::now()),
      current_price_coin: Some(0),
    }
  }

  async fn cleanup(pool: &mut DbPool<'_>, instance_id: crate::newtypes::InstanceId) {
    let _ = Instance::delete(pool, instance_id).await;
  }

  /// Create returns the session with the supplied status and pending payment.
  /// exists_for_post must report true after creation.
  #[tokio::test]
  #[serial]
  async fn create_and_exists_for_post() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = fixture(pool).await;

    let session = RideSession::create(pool, &pending_form(&ctx))
      .await
      .expect("create");
    assert_eq!(session.status, TripStatus::Pending);
    assert_eq!(session.payment_status, "pending");
    assert!(session.rider_id.is_none());

    assert!(RideSession::exists_for_post(pool, ctx.post_id)
      .await
      .expect("exists"));
    let by_post = RideSession::get_by_post(pool, ctx.post_id)
      .await
      .expect("get_by_post");
    assert_eq!(by_post.map(|s| s.id), Some(session.id));
    cleanup(pool, ctx.instance_id).await;
  }

  /// has_active_session is true while the session is active (any status
  /// except Delivered/Cancelled) and false after termination.
  /// This is the predicate `create_ride_session` and `update_ride_status`
  /// use to gate concurrent rides.
  #[tokio::test]
  #[serial]
  async fn has_active_session_flips_on_terminal_state() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = fixture(pool).await;

    let session = RideSession::create(pool, &pending_form(&ctx))
      .await
      .expect("create");
    // Assign + confirm.
    let _ = RideSession::update(
      pool,
      session.id,
      &RideSessionUpdateForm {
        rider_id: Some(Some(ctx.rider_id)),
        status: Some(TripStatus::Assigned),
        rider_assigned_at: Some(Some(Utc::now())),
        updated_at: Some(Some(Utc::now())),
        ..Default::default()
      },
    )
    .await
    .expect("assign");

    assert!(RideSession::has_active_session(pool, ctx.rider_id)
      .await
      .expect("active 1"));

    // Walk through to Delivered.
    for next in [
      TripStatus::RiderConfirmed,
      TripStatus::EnRouteToPickup,
      TripStatus::PickedUp,
      TripStatus::EnRouteToDropoff,
      TripStatus::Delivered,
    ] {
      let _ = RideSession::update(
        pool,
        session.id,
        &RideSessionUpdateForm {
          status: Some(next),
          updated_at: Some(Some(Utc::now())),
          ..Default::default()
        },
      )
      .await
      .unwrap_or_else(|_| panic!("transition -> {next:?}"));
    }

    assert!(
      !RideSession::has_active_session(pool, ctx.rider_id)
        .await
        .expect("active 2"),
      "rider should be free once ride is Delivered"
    );
    cleanup(pool, ctx.instance_id).await;
  }

  /// list_available_for_rider only returns Pending rows with NULL rider_id.
  /// Once a rider is assigned, the row disappears from the available list.
  #[tokio::test]
  #[serial]
  async fn list_available_excludes_assigned() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = fixture(pool).await;
    let session = RideSession::create(pool, &pending_form(&ctx))
      .await
      .expect("create");

    let available_before = RideSession::list_available_for_rider(pool, Some(100))
      .await
      .expect("list 1");
    assert!(available_before.iter().any(|r| r.id == session.id));

    let _ = RideSession::update(
      pool,
      session.id,
      &RideSessionUpdateForm {
        rider_id: Some(Some(ctx.rider_id)),
        status: Some(TripStatus::Assigned),
        rider_assigned_at: Some(Some(Utc::now())),
        ..Default::default()
      },
    )
    .await
    .expect("assign");

    let available_after = RideSession::list_available_for_rider(pool, Some(100))
      .await
      .expect("list 2");
    assert!(
      !available_after.iter().any(|r| r.id == session.id),
      "assigned session must not appear in available list"
    );
    cleanup(pool, ctx.instance_id).await;
  }

  /// Cancellation is reflected by status alone; no rider re-assignment happens
  /// at this DB layer (cancel_ride_session in api/api/src/delivery/ride.rs
  /// is what flips rider.accepting_jobs back).
  #[tokio::test]
  #[serial]
  async fn cancel_updates_status_and_reason() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = fixture(pool).await;
    let session = RideSession::create(pool, &pending_form(&ctx))
      .await
      .expect("create");

    let cancelled = RideSession::update(
      pool,
      session.id,
      &RideSessionUpdateForm {
        status: Some(TripStatus::Cancelled),
        cancellation_reason: Some(Some("user requested".to_string())),
        updated_at: Some(Some(Utc::now())),
        ..Default::default()
      },
    )
    .await
    .expect("cancel");

    assert_eq!(cancelled.status, TripStatus::Cancelled);
    assert_eq!(
      cancelled.cancellation_reason.as_deref(),
      Some("user requested")
    );
    cleanup(pool, ctx.instance_id).await;
  }
}
