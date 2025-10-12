use crate::{
  diesel::{BoolExpressionMethods, NullableExpressionMethods, OptionalExtension},
  newtypes::{CommunityId, DbUrl, InstanceId, LocalUserId, PersonId},
  source::person::{
    Person, PersonActions, PersonBlockForm, PersonFollowerForm, PersonInsertForm, PersonNoteForm,
    PersonUpdateForm,
  },
  traits::{ApubActor, Blockable, Crud, Followable},
  utils::{functions::lower, get_conn, uplete, DbPool},
};
use chrono::Utc;
use diesel::{
  dsl::{exists, insert_into, not, select},
  expression::SelectableHelper,
  ExpressionMethods, JoinOnDsl, QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{
  instance, instance_actions, local_user, person, person_actions,
};
use lemmy_utils::{
  error::{FastJobErrorExt, FastJobErrorType, FastJobResult},
  settings::structs::Settings,
};
use url::Url;

impl Crud for Person {
  type InsertForm = PersonInsertForm;
  type UpdateForm = PersonUpdateForm;
  type IdType = PersonId;

  // Override this, so that you don't get back deleted
  async fn read(pool: &mut DbPool<'_>, person_id: PersonId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    person::table
      .filter(person::deleted.eq(false))
      .find(person_id)
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  async fn create(pool: &mut DbPool<'_>, form: &PersonInsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreatePerson)
  }
  async fn update(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    form: &PersonUpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(person::table.find(person_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdatePerson)
  }
}

impl Person {
  /// Update or insert the person.
  ///
  /// actions.
  pub async fn create(pool: &mut DbPool<'_>, form: &PersonInsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdatePerson)
  }
  pub async fn upsert(pool: &mut DbPool<'_>, form: &PersonInsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person::table)
      .values(form)
      .on_conflict(person::ap_id)
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdatePerson)
  }

  pub async fn update_share_key(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    share_key: &str,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    diesel::update(person::table.find(person_id))
      .set(person::share_key.eq::<String>(share_key.to_owned()))
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateUser)
  }
  pub async fn delete_account(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    local_instance_id: InstanceId,
  ) -> FastJobResult<Person> {
    let conn = &mut get_conn(pool).await?;

    // Set the local user multilang to none, only if they aren't banned locally.
    let instance_actions_join = instance_actions::table.on(
      instance_actions::person_id
        .eq(person_id)
        .and(instance_actions::instance_id.eq(local_instance_id)),
    );

    let not_banned_local_user_id = local_user::table
      .left_join(instance_actions_join)
      .filter(local_user::person_id.eq(person_id))
      .filter(instance_actions::received_ban_at.nullable().is_null())
      .select(local_user::id)
      .first::<LocalUserId>(conn)
      .await
      .optional()?;

    if let Some(local_user_id) = not_banned_local_user_id {
      diesel::update(local_user::table.find(local_user_id))
        .set(local_user::email.eq::<Option<String>>(None))
        .execute(conn)
        .await?;
    };

    diesel::update(person::table.find(person_id))
      .set((
        person::display_name.eq::<Option<String>>(None),
        person::avatar.eq::<Option<String>>(None),
        person::banner.eq::<Option<String>>(None),
        person::bio.eq::<Option<String>>(None),
        person::matrix_user_id.eq::<Option<String>>(None),
        person::deleted.eq(true),
        person::updated_at.eq(Utc::now()),
      ))
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdatePerson)
  }

  pub async fn check_username_taken(pool: &mut DbPool<'_>, username: &str) -> FastJobResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(not(exists(
      person::table
        .filter(lower(person::name).eq(username.to_lowercase()))
        .filter(person::local.eq(true)),
    )))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(FastJobErrorType::UsernameAlreadyExists.into())
  }

  pub async fn read_by_name(pool: &mut DbPool<'_>, username: &str) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    person::table
      .filter(lower(person::name).eq(username.to_lowercase()))
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  // New helper: read person either by username (case-insensitive) or by LocalUser id (numeric string)
  pub async fn read_by_name_or_local_user_id(
    pool: &mut DbPool<'_>,
    username_or_id: &str,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    // Try by local_user.id if input looks like an integer
    if let Ok(local_user_id_num) = username_or_id.parse::<i32>() {
      use lemmy_db_schema_file::schema::local_user as lu;
      if let Some(found) = person::table
        .inner_join(lu::table.on(lu::person_id.eq(person::id)))
        .filter(lu::id.eq(local_user_id_num))
        .select(person::all_columns)
        .first::<Self>(conn)
        .await
        .optional()
        .with_fastjob_type(FastJobErrorType::NotFound)?
      {
        return Ok(found);
      }
    }

    // Fallback to name lookup
    person::table
      .filter(lower(person::name).eq(username_or_id.to_lowercase()))
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}

impl PersonInsertForm {
  pub fn test_form(instance_id: InstanceId, name: &str) -> Self {
    Self::new(name.to_owned(), "pubkey".to_string(), instance_id)
  }
}

impl ApubActor for Person {
  async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: &DbUrl,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    person::table
      .filter(person::deleted.eq(false))
      .filter(person::ap_id.eq(object_id))
      .first(conn)
      .await
      .optional()
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
  async fn read_from_name(
    pool: &mut DbPool<'_>,
    from_name: &str,
    include_deleted: bool,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    let mut q = person::table
      .into_boxed()
      .filter(person::local.eq(true))
      .filter(lower(person::name).eq(from_name.to_lowercase()));
    if !include_deleted {
      q = q.filter(person::deleted.eq(false))
    }
    q.first(conn)
      .await
      .optional()
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  async fn read_from_name_and_domain(
    pool: &mut DbPool<'_>,
    person_name: &str,
    for_domain: &str,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;

    person::table
      .inner_join(instance::table)
      .filter(lower(person::name).eq(person_name.to_lowercase()))
      .filter(lower(instance::domain).eq(for_domain.to_lowercase()))
      .select(person::all_columns)
      .first(conn)
      .await
      .optional()
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  fn generate_local_actor_url(name: &str, settings: &Settings) -> FastJobResult<DbUrl> {
    let domain = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!("{domain}/u/{name}"))?.into())
  }

  fn actor_url(&self, _settings: &Settings) -> FastJobResult<Url> {
    todo!()
  }
}

impl Followable for PersonActions {
  type Form = PersonFollowerForm;
  type IdType = PersonId;

  async fn follow(pool: &mut DbPool<'_>, form: &PersonFollowerForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person_actions::table)
      .values(form)
      .on_conflict((person_actions::person_id, person_actions::target_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CommunityFollowerAlreadyExists)
  }

  /// Currently no user following
  async fn follow_accepted(_: &mut DbPool<'_>, _: CommunityId, _: PersonId) -> FastJobResult<Self> {
    Err(FastJobErrorType::NotFound.into())
  }

  async fn unfollow(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    target_id: Self::IdType,
  ) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(person_actions::table.find((person_id, target_id)))
      .set_null(person_actions::followed_at)
      .set_null(person_actions::follow_pending)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CommunityFollowerAlreadyExists)
  }
}

impl Blockable for PersonActions {
  type Form = PersonBlockForm;
  type ObjectIdType = PersonId;
  type ObjectType = Person;

  async fn block(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person_actions::table)
      .values(form)
      .on_conflict((person_actions::person_id, person_actions::target_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::PersonBlockAlreadyExists)
  }

  async fn unblock(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(person_actions::table.find((form.person_id, form.target_id)))
      .set_null(person_actions::blocked_at)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::PersonBlockAlreadyExists)
  }

  async fn read_block(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    recipient_id: Self::ObjectIdType,
  ) -> FastJobResult<()> {
    let conn = &mut get_conn(pool).await?;
    let find_action = person_actions::table
      .find((person_id, recipient_id))
      .filter(person_actions::blocked_at.is_not_null());

    select(not(exists(find_action)))
      .get_result::<bool>(conn)
      .await?
      .then_some(())
      .ok_or(FastJobErrorType::PersonIsBlocked.into())
  }

  async fn read_blocks_for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> FastJobResult<Vec<Self::ObjectType>> {
    let conn = &mut get_conn(pool).await?;
    let target_person_alias = diesel::alias!(person as person1);

    person_actions::table
      .filter(person_actions::blocked_at.is_not_null())
      .inner_join(person::table.on(person_actions::person_id.eq(person::id)))
      .inner_join(
        target_person_alias.on(person_actions::target_id.eq(target_person_alias.field(person::id))),
      )
      .select(target_person_alias.fields(person::all_columns))
      .filter(person_actions::person_id.eq(person_id))
      .filter(target_person_alias.field(person::deleted).eq(false))
      .order_by(person_actions::blocked_at)
      .load::<Person>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}

impl PersonActions {
  pub async fn follower_inboxes(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
  ) -> FastJobResult<Vec<DbUrl>> {
    let conn = &mut get_conn(pool).await?;
    person_actions::table
      .filter(person_actions::followed_at.is_not_null())
      .inner_join(person::table.on(person_actions::person_id.eq(person::id)))
      .filter(person_actions::target_id.eq(for_person_id))
      .select(person::inbox_url)
      .distinct()
      .load(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn note(pool: &mut DbPool<'_>, form: &PersonNoteForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person_actions::table)
      .values(form)
      .on_conflict((person_actions::person_id, person_actions::target_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn delete_note(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    target_id: PersonId,
  ) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(person_actions::table.find((person_id, target_id)))
      .set_null(person_actions::note)
      .set_null(person_actions::noted_at)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn like(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    target_id: PersonId,
    like_score: i16,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    let (upvotes_inc, downvotes_inc) = match like_score {
      1 => (1, 0),
      -1 => (0, 1),
      _ => return Err(FastJobErrorType::NotFound.into()),
    };

    let voted_at = Utc::now();

    insert_into(person_actions::table)
      .values((
        person_actions::person_id.eq(person_id),
        person_actions::target_id.eq(target_id),
        person_actions::voted_at.eq(voted_at),
        person_actions::upvotes.eq(upvotes_inc),
        person_actions::downvotes.eq(downvotes_inc),
      ))
      .on_conflict((person_actions::person_id, person_actions::target_id))
      .do_update()
      .set((
        person_actions::person_id.eq(person_id),
        person_actions::target_id.eq(target_id),
        person_actions::voted_at.eq(voted_at),
        person_actions::upvotes.eq(person_actions::upvotes + upvotes_inc),
        person_actions::downvotes.eq(person_actions::downvotes + downvotes_inc),
      ))
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  /// Removes a person like. A previous_score of zero throws an error.
  pub async fn remove_like(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    target_id: PersonId,
    previous_score: i16,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    let (upvotes_inc, downvotes_inc) = match previous_score {
      1 => (-1, 0),
      -1 => (0, -1),
      _ => return Err(FastJobErrorType::NotFound.into()),
    };
    let voted_at = Utc::now();

    insert_into(person_actions::table)
      .values((
        person_actions::person_id.eq(person_id),
        person_actions::target_id.eq(target_id),
        person_actions::voted_at.eq(voted_at),
        person_actions::upvotes.eq(upvotes_inc),
        person_actions::downvotes.eq(downvotes_inc),
      ))
      .on_conflict((person_actions::person_id, person_actions::target_id))
      .do_update()
      .set((
        person_actions::person_id.eq(person_id),
        person_actions::target_id.eq(target_id),
        person_actions::voted_at.eq(voted_at),
        person_actions::upvotes.eq(person_actions::upvotes + upvotes_inc),
        person_actions::downvotes.eq(person_actions::downvotes + downvotes_inc),
      ))
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    source::{
      instance::Instance,
      person::{Person, PersonActions, PersonFollowerForm, PersonInsertForm},
    },
    traits::{Crud, Followable},
    utils::{build_db_pool_for_tests, uplete},
  };
  use lemmy_utils::error::FastJobResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn follow() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let person_form_1 = PersonInsertForm::test_form(inserted_instance.id, "erich");
    let person_1 = Person::create(pool, &person_form_1).await?;
    let person_form_2 = PersonInsertForm::test_form(inserted_instance.id, "michele");
    let person_2 = Person::create(pool, &person_form_2).await?;

    let follow_form = PersonFollowerForm::new(person_1.id, person_2.id, false);
    let person_follower = PersonActions::follow(pool, &follow_form).await?;
    assert_eq!(person_1.id, person_follower.target_id);
    assert_eq!(person_2.id, person_follower.person_id);
    assert!(person_follower.follow_pending.is_some_and(|x| !x));

    let followers = PersonActions::follower_inboxes(pool, person_1.id).await?;
    assert_eq!(vec![person_2.inbox_url], followers);

    let unfollow =
      PersonActions::unfollow(pool, follow_form.person_id, follow_form.target_id).await?;
    assert_eq!(uplete::Count::only_deleted(1), unfollow);

    Ok(())
  }
}
