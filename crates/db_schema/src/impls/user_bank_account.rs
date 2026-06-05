#[cfg(feature = "full")]
use crate::{
  newtypes::BankAccountId,
  source::user_bank_account::{BankAccount, UserBankAccountInsertForm, UserBankAccountUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use chrono::Utc;
use diesel::dsl::count_star;
use diesel::ExpressionMethods;

use crate::newtypes::{BankId, LocalUserId};
#[cfg(feature = "full")]
use app_108jobs_db_schema_file::schema::user_bank_accounts;
#[cfg(feature = "full")]
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
#[cfg(feature = "full")]
use diesel::QueryDsl;
use diesel_async::scoped_futures::ScopedFutureExt;
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;

#[cfg(feature = "full")]
impl Crud for BankAccount {
  type InsertForm = UserBankAccountInsertForm;
  type UpdateForm = UserBankAccountUpdateForm;
  type IdType = BankAccountId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(user_bank_accounts::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      // No specific error type exists for bank accounts; use a generic database error wrapper.
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(user_bank_accounts::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }
}

impl BankAccount {
  pub async fn exists_for_user_by_bank_and_number(
    pool: &mut DbPool<'_>,
    user_id: &LocalUserId,
    bank_id: &BankId,
    account_number: &str,
    exclude_id: Option<BankAccountId>,
  ) -> FastJobResult<bool> {
    let conn = &mut get_conn(pool).await?;

    let mut query = user_bank_accounts::table
      .filter(user_bank_accounts::local_user_id.eq(user_id))
      .filter(user_bank_accounts::bank_id.eq(bank_id))
      .filter(user_bank_accounts::account_number.eq(account_number.to_string()))
      .into_boxed();

    if let Some(ex_id) = exclude_id {
      query = query.filter(user_bank_accounts::id.ne(ex_id));
    }

    let count: i64 = query.select(count_star()).get_result(conn).await?;

    Ok(count > 0)
  }

  pub async fn set_default(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    bank_account_id: BankAccountId,
  ) -> FastJobResult<BankAccount> {
    let conn = &mut get_conn(pool).await?;

    // 1-step flow inside one transaction
    let updated = conn
      .run_transaction(|conn| {
        async move {
          let now_time = Utc::now();

          // 1) unset all defaults for this user
          diesel::update(
            user_bank_accounts::table.filter(user_bank_accounts::local_user_id.eq(user_id)),
          )
          .set((
            user_bank_accounts::is_default.eq(false),
            user_bank_accounts::updated_at.eq(now_time),
          ))
          .execute(conn)
          .await
          .with_fastjob_type(FastJobErrorType::CouldntUpdateBankAccount)?;

          // 2) set requested account to default and return it
          let updated_acc = diesel::update(
            user_bank_accounts::table
              .filter(user_bank_accounts::id.eq(bank_account_id))
              .filter(user_bank_accounts::local_user_id.eq(user_id)),
          )
          .set((
            user_bank_accounts::is_default.eq(true),
            user_bank_accounts::updated_at.eq(now_time),
          ))
          .get_result::<BankAccount>(conn)
          .await
          .with_fastjob_type(FastJobErrorType::CouldntUpdateBankAccount)?;

          Ok(updated_acc)
        }
        .scope_boxed()
      })
      .await?;
    Ok(updated)
  }

  pub async fn count_for_user(pool: &mut DbPool<'_>, user_id: &LocalUserId) -> FastJobResult<i64> {
    let conn = &mut get_conn(pool).await?;

    let count: i64 = user_bank_accounts::table
      .filter(user_bank_accounts::local_user_id.eq(user_id))
      .select(count_star())
      .get_result(conn)
      .await?;

    Ok(count)
  }
}

// ============================================================================
// DB-backed tests for the user bank account CRUD surface used by
// `api/local_user/bank_account` and `api/admin/bank_account`.
//
// Coverage:
//   * set_default keeps the "only one default per user" invariant atomically
//   * set_default does NOT cross tenant boundaries
//   * admin_verify_bank_account is idempotent (re-applying the same is_verified
//     update yields the same row)
//   * exists_for_user_by_bank_and_number powers the create-time uniqueness check
// ============================================================================
#[cfg(feature = "full")]
#[cfg(test)]
mod tests {
  use super::*;
  use crate::source::bank::BankInsertForm;
  use crate::source::instance::Instance;
  use crate::source::person::{Person, PersonInsertForm};
  use crate::test_data::pool_for_tests;
  use app_108jobs_db_schema_file::schema::{banks, local_user};
  use diesel::ExpressionMethods;
  use diesel_async::RunQueryDsl;
  use serial_test::serial;

  struct UserCtx {
    instance_id: crate::newtypes::InstanceId,
    local_user_id: LocalUserId,
    bank_id: BankId,
  }

  async fn make_user(pool: &mut DbPool<'_>, label: &str) -> UserCtx {
    let inst = Instance::read_or_create(
      pool,
      format!("ba-test-{}-{}.tld", label, uuid::Uuid::new_v4()),
    )
    .await
    .expect("create instance");

    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let suffix_short = &suffix[..8];
    let (p_form, _wallet) =
      PersonInsertForm::test_form_with_wallet(pool, inst.id, &format!("{label}-{suffix_short}"))
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

    let bank_form = BankInsertForm {
      name: format!("BA Bank {suffix_short}"),
      country_id: "TH".to_string(),
      bank_code: Some(format!("B{suffix_short}")),
      swift_code: None,
      is_active: Some(true),
    };
    let bank_id: i32 = diesel::insert_into(banks::table)
      .values(&bank_form)
      .returning(banks::id)
      .get_result(conn)
      .await
      .expect("insert bank");

    UserCtx {
      instance_id: inst.id,
      local_user_id: LocalUserId(local_user_id),
      bank_id: BankId(bank_id),
    }
  }

  async fn create_account(
    pool: &mut DbPool<'_>,
    ctx: &UserCtx,
    account_number: &str,
  ) -> BankAccount {
    BankAccount::create(
      pool,
      &UserBankAccountInsertForm {
        local_user_id: ctx.local_user_id,
        bank_id: ctx.bank_id,
        account_number: account_number.to_string(),
        account_name: format!("Holder {}", account_number),
        verification_image_path: None,
      },
    )
    .await
    .expect("create bank account")
  }

  async fn read_account(pool: &mut DbPool<'_>, id: BankAccountId) -> BankAccount {
    use diesel::SelectableHelper;
    let conn = &mut get_conn(pool).await.expect("conn");
    user_bank_accounts::table
      .find(id)
      .select(BankAccount::as_select())
      .first::<BankAccount>(conn)
      .await
      .unwrap_or_else(|e| panic!("re-read bank account {id:?}: {e:?}"))
  }

  async fn cleanup(pool: &mut DbPool<'_>, instance_id: crate::newtypes::InstanceId) {
    let _ = Instance::delete(pool, instance_id).await;
  }

  /// Setting a new default unsets the previous one in the same transaction.
  /// Verifies the `set_default` invariant: at most one `is_default=true` per user.
  #[tokio::test]
  #[serial]
  async fn set_default_keeps_single_default_per_user() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = make_user(pool, "u").await;

    let a = create_account(pool, &ctx, "111").await;
    let b = create_account(pool, &ctx, "222").await;

    // initially nothing is default
    assert!(!read_account(pool, a.id).await.is_default);
    assert!(!read_account(pool, b.id).await.is_default);

    let _ = BankAccount::set_default(pool, ctx.local_user_id, a.id)
      .await
      .expect("set a default");
    assert!(read_account(pool, a.id).await.is_default);
    assert!(!read_account(pool, b.id).await.is_default);

    let _ = BankAccount::set_default(pool, ctx.local_user_id, b.id)
      .await
      .expect("set b default");
    assert!(!read_account(pool, a.id).await.is_default);
    assert!(read_account(pool, b.id).await.is_default);
    cleanup(pool, ctx.instance_id).await;
  }

  /// set_default scoped to the calling user must not touch another user's rows.
  #[tokio::test]
  #[serial]
  async fn set_default_is_user_scoped() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let alice = make_user(pool, "alice").await;
    let bob = make_user(pool, "bob").await;

    let alice_acc = create_account(pool, &alice, "aa").await;
    let bob_acc = create_account(pool, &bob, "bb").await;
    let _ = BankAccount::set_default(pool, bob.local_user_id, bob_acc.id)
      .await
      .expect("bob default");

    let _ = BankAccount::set_default(pool, alice.local_user_id, alice_acc.id)
      .await
      .expect("alice default");

    // Bob's default must not have been unset by Alice's call.
    assert!(read_account(pool, bob_acc.id).await.is_default);
    assert!(read_account(pool, alice_acc.id).await.is_default);

    cleanup(pool, alice.instance_id).await;
    cleanup(pool, bob.instance_id).await;
  }

  /// admin_verify_bank_account toggles is_verified to true and is idempotent.
  #[tokio::test]
  #[serial]
  async fn verify_is_idempotent() {
    use crate::traits::Crud;
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = make_user(pool, "v").await;
    let acc = create_account(pool, &ctx, "999").await;
    assert!(!acc.is_verified);

    let v1 = BankAccount::update(
      pool,
      acc.id,
      &UserBankAccountUpdateForm {
        is_verified: Some(true),
        updated_at: Some(Some(chrono::Utc::now())),
        ..Default::default()
      },
    )
    .await
    .expect("first verify");
    assert!(v1.is_verified);

    let v2 = BankAccount::update(
      pool,
      acc.id,
      &UserBankAccountUpdateForm {
        is_verified: Some(true),
        updated_at: Some(Some(chrono::Utc::now())),
        ..Default::default()
      },
    )
    .await
    .expect("second verify");
    assert!(v2.is_verified);
    assert_eq!(v1.id, v2.id);
    cleanup(pool, ctx.instance_id).await;
  }

  /// Duplicate detection respects (user, bank, account_number) and excludes
  /// a known id (used during update flows).
  #[tokio::test]
  #[serial]
  async fn exists_check_respects_user_bank_number() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = make_user(pool, "e").await;
    let acc = create_account(pool, &ctx, "555").await;

    let exists = BankAccount::exists_for_user_by_bank_and_number(
      pool,
      &ctx.local_user_id,
      &ctx.bank_id,
      "555",
      None,
    )
    .await
    .expect("exists");
    assert!(exists, "freshly inserted row must be detected");

    let exists_excl = BankAccount::exists_for_user_by_bank_and_number(
      pool,
      &ctx.local_user_id,
      &ctx.bank_id,
      "555",
      Some(acc.id),
    )
    .await
    .expect("exists excl");
    assert!(!exists_excl, "self-exclude should not collide");

    let exists_other = BankAccount::exists_for_user_by_bank_and_number(
      pool,
      &ctx.local_user_id,
      &ctx.bank_id,
      "different",
      None,
    )
    .await
    .expect("exists other");
    assert!(!exists_other);
    cleanup(pool, ctx.instance_id).await;
  }
}
