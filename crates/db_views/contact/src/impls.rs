use crate::ContactView;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{LocalUserId},
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::contact;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl ContactView {

  pub async fn find_by_local_user_id(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    contact::table
      .filter(contact::local_user_id.eq(local_user_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntFindContact)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use lemmy_db_schema::{
    source::{
      contact::{Contact, ContactInsertForm},
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use lemmy_utils::error::FastJobResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  struct Data {
    contact: Contact,
    local_user: LocalUser,
    person: Person,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> FastJobResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let person_form = PersonInsertForm {
      local: Some(true),
      ..PersonInsertForm::test_form(instance.id, "alice")
    };
    let person = Person::create(pool, &person_form).await?;
    let local_user_form = LocalUserInsertForm::test_form(person.id);
    let local_user = LocalUser::create(pool, &local_user_form, vec![]).await?;

    let contact_form = ContactInsertForm {
      local_user_id: local_user.id,
      phone: Some("1234567890".to_string()),
      email: Some("test@example.com".to_string()),
      secondary_email: None,
      line_id: None,
      facebook: None,
    };
    let contact = Contact::create(pool, &contact_form).await?;

    Ok(Data {
      contact,
      local_user,
      person,
    })
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> FastJobResult<()> {
    Contact::delete(pool, data.contact.id).await?;
    LocalUser::delete(pool, data.local_user.id).await?;
    Person::delete(pool, data.person.id).await?;
    Instance::delete(pool, data.person.instance_id).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_read_contact() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let contact_view = ContactView::read(pool, data.contact.id).await?;
    assert_eq!(contact_view.contact.id, data.contact.id);
    assert_eq!(contact_view.contact.local_user_id, data.local_user.id);
    assert_eq!(contact_view.contact.phone, Some("1234567890".to_string()));
    assert_eq!(contact_view.contact.email, Some("test@example.com".to_string()));

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_find_by_local_user_id() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let contact_view = ContactView::find_by_local_user_id(pool, data.local_user.id).await?;
    assert_eq!(contact_view.contact.id, data.contact.id);
    assert_eq!(contact_view.contact.local_user_id, data.local_user.id);

    cleanup(data, pool).await
  }
}