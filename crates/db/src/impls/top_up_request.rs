use crate::{
  newtypes::TopUpRequestId,
  source::top_up_request::{TopUpRequest, TopUpRequestInsertForm, TopUpRequestUpdateForm},
};
#[cfg(feature = "full")]
use crate::{
  traits::Crud,
  utils::{get_conn, DbPool},
};
#[cfg(feature = "full")]
use app_108jobs_core::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
#[cfg(feature = "full")]
use crate::schema::top_up_requests;
use diesel::ExpressionMethods;
#[cfg(feature = "full")]
use diesel::QueryDsl;
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;

#[cfg(feature = "full")]
impl Crud for TopUpRequest {
  type InsertForm = TopUpRequestInsertForm;
  type UpdateForm = TopUpRequestUpdateForm;
  type IdType = TopUpRequestId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(top_up_requests::table)
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
    diesel::update(top_up_requests::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }
}

#[cfg(feature = "full")]
impl TopUpRequest {
  pub async fn get_by_qr_id(pool: &mut DbPool<'_>, qr_id: &str) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    top_up_requests::table
      .filter(top_up_requests::qr_id.eq(qr_id))
      .first::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  pub async fn update_by_qr_id(
    pool: &mut DbPool<'_>,
    qr_id: String,
    form: &TopUpRequestUpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(top_up_requests::table.filter(top_up_requests::qr_id.eq(qr_id)))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  /// Re-read a `TopUpRequest` by `qr_id` with `SELECT ... FOR UPDATE` on the
  /// supplied connection. Intended for use inside a `run_transaction` so the
  /// caller can serialize concurrent admin top-up calls on the same row and
  /// inspect the `transferred` flag race-safely before crediting.
  pub async fn lock_for_credit_on_conn(
    conn: &mut diesel_async::AsyncPgConnection,
    qr_id: &str,
  ) -> FastJobResult<Self> {
    top_up_requests::table
      .filter(top_up_requests::qr_id.eq(qr_id))
      .for_update()
      .first::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  /// Flip `transferred = true` for the row identified by `qr_id` on the
  /// supplied connection. Returns `Ok(())` if exactly one row updated,
  /// `Err(NotFound)` otherwise. Intended for use inside the same
  /// `run_transaction` that holds the `FOR UPDATE` lock from
  /// [`Self::lock_for_credit_on_conn`].
  pub async fn mark_transferred_on_conn(
    conn: &mut diesel_async::AsyncPgConnection,
    qr_id: &str,
  ) -> FastJobResult<()> {
    let updated = diesel::update(top_up_requests::table.filter(top_up_requests::qr_id.eq(qr_id)))
      .set((
        top_up_requests::transferred.eq(true),
        top_up_requests::updated_at.eq(chrono::Utc::now()),
      ))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;
    if updated == 0 {
      return Err(FastJobErrorType::NotFound.into());
    }
    Ok(())
  }
}

// ============================================================================
// DB-backed tests for the TopUpRequest CRUD surface used by
// `routes/payments/{create_qrcode, inquire}` and `api/admin/wallet::top_up`.
//
// Coverage:
//   * status flow Pending -> Success advances via update_by_qr_id
//   * `transferred` flag toggles independently of status (audited path)
//   * get_by_qr_id is the lookup admin_top_up_wallet relies on
//
// The flows that depend on these primitives (admin_top_up_wallet,
// inquire_qrcode) cannot be tested at this layer because they wrap a real
// reqwest::Client to SCB — see the report at the end of this turn for the
// not-confirmed gap.
// ============================================================================
#[cfg(feature = "full")]
#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    newtypes::{Coin, LocalUserId},
    source::{
      currency::Currency,
      instance::Instance,
      person::{Person, PersonInsertForm},
    },
    test_data::pool_for_tests,
    utils::get_conn,
  };
  use crate::{enums::TopUpStatus, schema::local_user};
  use chrono::{Duration, Utc};
  use diesel::ExpressionMethods;
  use diesel_async::RunQueryDsl;
  use serial_test::serial;

  /// Minimal fixture: instance, one user with wallet, default currency
  /// (seeded by migration `2026-02-06-140000-0000_add_currency_support`).
  async fn fixture(
    pool: &mut DbPool<'_>,
  ) -> (
    crate::newtypes::InstanceId,
    LocalUserId,
    crate::newtypes::CurrencyId,
  ) {
    let inst = Instance::read_or_create(pool, format!("topup-test-{}.tld", uuid::Uuid::new_v4()))
      .await
      .expect("create instance");

    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let suffix_short = &suffix[..8];
    let (p_form, _wallet) =
      PersonInsertForm::test_form_with_wallet(pool, inst.id, &format!("top-{suffix_short}"))
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

    let currency = Currency::get_default(pool)
      .await
      .expect("get_default currency")
      .expect("THB seeded by migration");

    (inst.id, LocalUserId(local_user_id), currency.id)
  }

  async fn cleanup(pool: &mut DbPool<'_>, instance_id: crate::newtypes::InstanceId) {
    let _ = Instance::delete(pool, instance_id).await;
  }

  fn insert_form(
    local_user_id: LocalUserId,
    currency_id: crate::newtypes::CurrencyId,
  ) -> TopUpRequestInsertForm {
    TopUpRequestInsertForm {
      local_user_id,
      amount: 100.0,
      currency_id,
      amount_coin: Coin(10_000),
      conversion_rate_used: 1,
      qr_id: format!("qr-{}", uuid::Uuid::new_v4()),
      cs_ext_expiry_time: Utc::now() + Duration::minutes(5),
      paid_at: None,
    }
  }

  /// Default state: status=Pending, transferred=false, paid_at=None.
  #[tokio::test]
  #[serial]
  async fn create_defaults_pending_not_transferred() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let (instance_id, user_id, currency_id) = fixture(pool).await;
    let form = insert_form(user_id, currency_id);
    let qr_id = form.qr_id.clone();

    let created = TopUpRequest::create(pool, &form).await.expect("create");
    assert_eq!(created.status, TopUpStatus::Pending);
    assert!(!created.transferred);
    assert!(created.paid_at.is_none());

    let fetched = TopUpRequest::get_by_qr_id(pool, &qr_id)
      .await
      .expect("get_by_qr_id");
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.amount_coin.0, 10_000);
    cleanup(pool, instance_id).await;
  }

  /// inquire_qrcode flips status Pending -> Success when SCB reports paid.
  #[tokio::test]
  #[serial]
  async fn update_by_qr_id_marks_success() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let (instance_id, user_id, currency_id) = fixture(pool).await;
    let form = insert_form(user_id, currency_id);
    let qr_id = form.qr_id.clone();
    let _created = TopUpRequest::create(pool, &form).await.expect("create");

    let now = Utc::now();
    let update = TopUpRequestUpdateForm {
      status: Some(TopUpStatus::Success),
      updated_at: Some(now),
      paid_at: Some(Some(now)),
      transferred: None,
    };
    let updated = TopUpRequest::update_by_qr_id(pool, qr_id.clone(), &update)
      .await
      .expect("update");
    assert_eq!(updated.status, TopUpStatus::Success);
    assert!(updated.paid_at.is_some());
    assert!(
      !updated.transferred,
      "transferred flips on credit, not on payment"
    );
    cleanup(pool, instance_id).await;
  }

  /// admin_top_up_wallet flips `transferred` AFTER crediting the wallet.
  /// `status` should be independent of `transferred` so the admin retry
  /// guard (`if topup_request.transferred { ... already processed ... }`)
  /// is the authoritative duplicate check.
  #[tokio::test]
  #[serial]
  async fn transferred_flag_independent_of_status() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let (instance_id, user_id, currency_id) = fixture(pool).await;
    let form = insert_form(user_id, currency_id);
    let qr_id = form.qr_id.clone();
    let _created = TopUpRequest::create(pool, &form).await.expect("create");

    // 1) inquire path: status -> Success
    let now = Utc::now();
    TopUpRequest::update_by_qr_id(
      pool,
      qr_id.clone(),
      &TopUpRequestUpdateForm {
        status: Some(TopUpStatus::Success),
        updated_at: Some(now),
        paid_at: Some(Some(now)),
        transferred: None,
      },
    )
    .await
    .expect("inquire update");

    // 2) admin credit path: only transferred -> true
    let after_credit = TopUpRequest::update_by_qr_id(
      pool,
      qr_id.clone(),
      &TopUpRequestUpdateForm {
        status: None,
        updated_at: Some(Utc::now()),
        paid_at: None,
        transferred: Some(true),
      },
    )
    .await
    .expect("admin credit update");

    assert_eq!(after_credit.status, TopUpStatus::Success);
    assert!(after_credit.transferred);
    cleanup(pool, instance_id).await;
  }
}
