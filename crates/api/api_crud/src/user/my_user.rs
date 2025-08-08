use actix_web::web::Data;
use actix_web::web::Json;
use lemmy_api_utils::{context::FastJobContext, utils::check_local_user_valid};
use lemmy_db_schema::{
  source::{
    actor_language::LocalUserLanguage,
    instance::InstanceActions,
    keyword_block::LocalUserKeywordBlock,
    person::PersonActions,
    contact::Contact,
    address::Address,
    identity_card::IdentityCard,
  },
  traits::{Blockable, Crud},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::MyUserInfo;
use lemmy_db_views_wallet::WalletView;
use lemmy_db_views_person::ProfileDataView;
use lemmy_utils::error::FastJobResult;

pub async fn get_my_user(
  local_user_view: LocalUserView,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<MyUserInfo>> {
  check_local_user_valid(&local_user_view)?;

  // Build the local user with parallel queries and add it to site response
  let person_id = local_user_view.person.id;
  let local_user_id = local_user_view.local_user.id;
  let pool = &mut context.pool();

  // Fetch non-profile data first
  let (
    instance_blocks,
    person_blocks,
    keyword_blocks,
    discussion_languages,
    wallet,
  ) = lemmy_db_schema::try_join_with_pool!(pool => (
    |pool| InstanceActions::read_blocks_for_person(pool, person_id),
    |pool| PersonActions::read_blocks_for_person(pool, person_id),
    |pool| LocalUserKeywordBlock::read(pool, local_user_id),
    |pool| LocalUserLanguage::read(pool, local_user_id),
    |pool| WalletView::read_by_user(pool, local_user_id)
  ))?;

  // Fetch profile data with error handling
  let contact_result = Contact::read(pool, local_user_view.person.contact_id).await;
  let address_result = Address::read(pool, local_user_view.person.address_id).await;
  let identity_card_result = IdentityCard::read(pool, local_user_view.person.identity_card_id).await;

  // Use the results if available, otherwise use default values
  let contact = contact_result.unwrap_or_else(|_| Contact {
    id: local_user_view.person.contact_id,
    phone: None,
    email: None,
    secondary_email: None,
    line_id: None,
    facebook: None,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  });

  let address = address_result.unwrap_or_else(|_| Address {
    id: local_user_view.person.address_id,
    address_line1: "".to_string(),
    address_line2: None,
    subdistrict: None,
    district: "".to_string(),
    province: "".to_string(),
    postal_code: "".to_string(),
    country_id: "".to_string(),
    is_default: false,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  });

  let identity_card = identity_card_result.unwrap_or_else(|_| IdentityCard {
    id: local_user_view.person.identity_card_id,
    address_id: local_user_view.person.address_id,
    id_number: "".to_string(),
    issued_date: chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
    expiry_date: chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
    full_name: "".to_string(),
    date_of_birth: chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
    nationality: "".to_string(),
    is_verified: false,
    created_at: chrono::Utc::now(),
  });

  let profile = ProfileDataView{
    contact,
    address,
    identity_card,
    show_country_selection_box: false,
    is_new_buyer: false,
  };
  Ok(Json(MyUserInfo {
    local_user_view: local_user_view.clone(),
    community_blocks: Vec::new(),
    instance_blocks,
    person_blocks,
    keyword_blocks,
    discussion_languages,
    wallet,
    profile,
  }))
}
