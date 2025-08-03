use crate::{
  newtypes::{InstanceId, PersonId},
  source::{
    instance::{Instance, InstanceActions, InstanceBanForm, InstanceBlockForm, InstanceForm},
  },
  traits::{Bannable, Blockable},
  utils::{
    functions::lower,
    get_conn,
    uplete,
    DbPool,
  },
};
use chrono::Utc;
use diesel::{
  dsl::{exists, insert_into, not, select},
  ExpressionMethods,
  OptionalExtension,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{
  instance,
  instance_actions,
};
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Instance {
  /// Attempt to read Instance column for the given domain. If it doesn't exist, insert a new one.
  /// There is no need for update as the domain of an existing instance cant change.
  pub async fn read_or_create(pool: &mut DbPool<'_>, domain_: String) -> FastJobResult<Self> {
    use lemmy_db_schema_file::schema::instance::domain;
    let conn = &mut get_conn(pool).await?;

    // First try to read the instance row and return directly if found
    let instance = instance::table
     .filter(lower(domain).eq(&domain_.to_lowercase()))
     .first(conn)
     .await
     .optional()?;

    // TODO could convert this to unwrap_or_else once async closures are stable
    match instance {
      Some(i) => Ok(i),
      None => {
        // Instance not in database yet, insert it
        let form = InstanceForm {
          updated_at: Some(Utc::now()),
          ..InstanceForm::new(domain_)
        };
        insert_into(instance::table)
         .values(&form)
         // Necessary because this method may be called concurrently for the same domain. This
         // could be handled with a transaction, but nested transactions arent allowed
         .on_conflict(instance::domain)
         .do_update()
         .set(&form)
         .get_result::<Self>(conn)
         .await
         .with_fastjob_type(FastJobErrorType::CouldntCreateSite)
      }
    }
  }
  pub async fn read(pool: &mut DbPool<'_>, instance_id: InstanceId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    instance::table
     .find(instance_id)
     .first(conn)
     .await
     .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn update(
    pool: &mut DbPool<'_>,
    instance_id: InstanceId,
    form: InstanceForm,
  ) -> FastJobResult<usize> {
    let mut conn = get_conn(pool).await?;
    diesel::update(instance::table.find(instance_id))
     .set(form)
     .execute(&mut conn)
     .await
     .with_fastjob_type(FastJobErrorType::CouldntUpdateSite)
  }

  pub async fn delete(pool: &mut DbPool<'_>, instance_id: InstanceId) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(instance::table.find(instance_id))
     .execute(conn)
     .await
     .with_fastjob_type(FastJobErrorType::Deleted)
  }

  pub async fn read_all(pool: &mut DbPool<'_>) -> FastJobResult<Vec<Instance>> {
    let conn = &mut get_conn(pool).await?;
    instance::table
     .select(Self::as_select())
     .get_results(conn)
     .await
     .with_fastjob_type(FastJobErrorType::NotFound)
  }

}
impl Blockable for InstanceActions {
  type Form = InstanceBlockForm;
  type ObjectIdType = InstanceId;
  type ObjectType = Instance;

  async fn block(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(instance_actions::table)
      .values(form)
      .on_conflict((instance_actions::person_id, instance_actions::instance_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::InstanceBlockAlreadyExists)
  }

  async fn unblock(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(instance_actions::table.find((form.person_id, form.instance_id)))
      .set_null(instance_actions::blocked_at)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::InstanceBlockAlreadyExists)
  }

  async fn read_block(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    instance_id: Self::ObjectIdType,
  ) -> FastJobResult<()> {
    let conn = &mut get_conn(pool).await?;
    let find_action = instance_actions::table
      .find((person_id, instance_id))
      .filter(instance_actions::blocked_at.is_not_null());
    select(not(exists(find_action)))
      .get_result::<bool>(conn)
      .await?
      .then_some(())
      .ok_or(FastJobErrorType::InstanceIsBlocked.into())
  }

  async fn read_blocks_for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> FastJobResult<Vec<Self::ObjectType>> {
    let conn = &mut get_conn(pool).await?;
    instance_actions::table
      .filter(instance_actions::blocked_at.is_not_null())
      .inner_join(instance::table)
      .select(instance::all_columns)
      .filter(instance_actions::person_id.eq(person_id))
      .order_by(instance_actions::blocked_at)
      .load::<Instance>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}

impl InstanceActions {
  pub async fn check_ban(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    instance_id: InstanceId,
  ) -> FastJobResult<()> {
    let conn = &mut get_conn(pool).await?;
    let ban_exists = select(exists(
      instance_actions::table
        .filter(instance_actions::person_id.eq(person_id))
        .filter(instance_actions::instance_id.eq(instance_id))
        .filter(instance_actions::received_ban_at.is_not_null()),
    ))
    .get_result::<bool>(conn)
    .await?;

    if ban_exists {
      return Err(FastJobErrorType::SiteBan.into());
    }
    Ok(())
  }
}

impl Bannable for InstanceActions {
  type Form = InstanceBanForm;
  async fn ban(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      insert_into(instance_actions::table)
        .values(form)
        .on_conflict((instance_actions::person_id, instance_actions::instance_id))
        .do_update()
        .set(form)
        .returning(Self::as_select())
        .get_result::<Self>(conn)
        .await?,
    )
  }
  async fn unban(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      uplete::new(instance_actions::table.find((form.person_id, form.instance_id)))
        .set_null(instance_actions::received_ban_at)
        .set_null(instance_actions::ban_expires_at)
        .get_result(conn)
        .await?,
    )
  }
}
