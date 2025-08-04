use actix_web::web::Data;
use actix_web::web::Json;
use lemmy_api_utils::{context::FastJobContext, utils::check_local_user_valid};
use lemmy_db_schema::{
  source::{
    actor_language::LocalUserLanguage,
    instance::InstanceActions,
    keyword_block::LocalUserKeywordBlock,
    person::PersonActions,
  },
  traits::Blockable,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::MyUserInfo;
use lemmy_db_views_wallet::WalletView;
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

  Ok(Json(MyUserInfo {
    local_user_view: local_user_view.clone(),
    community_blocks: Vec::new(),
    instance_blocks,
    person_blocks,
    keyword_blocks,
    discussion_languages,
    wallet: wallet.map(|w| w.wallet),
  }))
}
