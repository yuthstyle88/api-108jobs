use chrono::{NaiveDate, Utc};
use diesel::{
  dsl::{exists, not, select},
  query_builder::AsQuery,
};
use diesel_async::{scoped_futures::ScopedFutureExt, RunQueryDsl};
use lemmy_api_utils::utils::generate_inbox_url;
use lemmy_db_schema::{
  source::{
    instance::Instance,
    local_site::{LocalSite, LocalSiteInsertForm},
    local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitInsertForm},
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm},
    site::{Site, SiteInsertForm},
  },
  traits::{ApubActor, Crud},
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::local_site;
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::{FastJobErrorExt, FastJobErrorType, FastJobResult},
  settings::structs::Settings,
};
use tracing::info;
use url::Url;
use lemmy_db_schema::source::address::{Address, AddressInsertForm};
use lemmy_db_schema::source::contact::{Contact, ContactInsertForm};
use lemmy_db_schema::source::identity_card::{IdentityCard, IdentityCardInsertForm};
use lemmy_db_schema_file::enums::Role;

pub async fn setup_local_site(pool: &mut DbPool<'_>, settings: &Settings) -> FastJobResult<SiteView> {
  let conn = &mut get_conn(pool).await?;
  // Check to see if local_site exists, without the cache wrapper
  if select(not(exists(local_site::table.as_query())))
    .get_result(conn)
    .await?
  {
    info!("No Local Site found, creating it.");

    let domain = settings
      .get_hostname_without_port()
      .with_fastjob_type(FastJobErrorType::Unknown("must have domain".into()))?;

    conn
      .run_transaction(|conn| {
        async move {
          // Upsert this to the instance table
          let instance = Instance::read_or_create(&mut conn.into(), domain).await?;

          if let Some(setup) = &settings.setup {
            let today_utc: NaiveDate = Utc::now().date_naive();
            let person_ap_id = Person::generate_local_actor_url(&setup.admin_username, settings)?;
            let public_key = "public_key".to_string();
            let private_key = Some("private_key".to_string());
            let form = AddressInsertForm{
              address_line1: "No".to_string(),
              address_line2: None,
              subdistrict: None,
              district: "Bang Ka Pi".to_string(),
              province: "Bangkok".to_string(),
              postal_code: "10240".to_string(),
              country_id: "TH".to_string(),
              is_default: Some(true),
            };
            let address_id = Address::create(&mut conn.into(), &form).await?.id;
            let form = ContactInsertForm{
              phone: Some("0812235666".to_string()),
              email: Some("noreply@fastjob.com".to_string()),
              secondary_email: None,
              line_id: None,
              facebook: None,
            };
            let contact_id = Contact::create(&mut conn.into(), &form).await?.id;
            let from = IdentityCardInsertForm{
              address_id,
              id_number: "8888888888888".to_string(),
              issued_date: today_utc,
              expiry_date: today_utc,
              full_name: "fast job".to_string(),
              date_of_birth: today_utc,
              nationality: "Thai".to_string(),
              is_verified: Some(true),
            };
            let identity_card_id = IdentityCard::create(&mut conn.into(), &from).await?.id;

            // Register the user if there's a site setup
            let person_form = PersonInsertForm {
              ap_id: Some(person_ap_id.clone()),
              inbox_url: Some(generate_inbox_url()?),
              private_key,
              address_id: Some(address_id),
              contact_id: Some(contact_id),
              identity_card_id: Some(identity_card_id),
              ..PersonInsertForm::new(
                setup.admin_username.clone(),
                public_key,
                instance.id,
              )
            };
            let person_inserted = Person::create(&mut conn.into(), &person_form).await?;

            let local_user_form = LocalUserInsertForm {
              email: setup.admin_email.clone(),
              admin: Some(true),
              role: Some(Role::Admin),
              ..LocalUserInsertForm::new(person_inserted.id, Some(setup.admin_password.clone()))
            };
            LocalUser::create(&mut conn.into(), &local_user_form, vec![]).await?;
          };

          // Add an entry for the site table
          let site_key_pair = "site_key_pair".to_string();
          let site_ap_id = Url::parse(&settings.get_protocol_and_hostname())?;

          let name = settings
            .setup
            .clone()
            .map(|s| s.site_name)
            .unwrap_or_else(|| "New Site".to_string());
          let site_form = SiteInsertForm {
            ap_id: Some(site_ap_id.clone().into()),
            last_refreshed_at: Some(Utc::now()),
            inbox_url: Some(generate_inbox_url()?),
            private_key: Some(site_key_pair.clone()),
            public_key: Some(site_key_pair),
             ..SiteInsertForm::new(name, instance.id)
          };
          let site = Site::create(&mut conn.into(), &site_form).await?;

          // Finally create the local_site row
          let local_site_form = LocalSiteInsertForm {
            site_setup: Some(settings.setup.is_some()),
            community_creation_admin_only: Some(true),
            ..LocalSiteInsertForm::new(site.id)
          };
          let local_site = LocalSite::create(&mut conn.into(), &local_site_form).await?;

          // Create the rate limit table
          let local_site_rate_limit_form = LocalSiteRateLimitInsertForm::new(local_site.id);
          LocalSiteRateLimit::create(&mut conn.into(), &local_site_rate_limit_form).await?;
          Ok(())
        }
        .scope_boxed()
      })
      .await?;
  }

  SiteView::read_local(pool).await
}
