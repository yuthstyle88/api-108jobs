use actix_web::web::{Data, Json};
use app_108jobs_api_utils::{context::FastJobContext, utils::check_local_user_valid};
use app_108jobs_core::error::FastJobResult;
use app_108jobs_db::{
  source::{
    actor_language::LocalUserLanguage,
    instance::InstanceActions,
    keyword_block::LocalUserKeywordBlock,
    person::PersonActions,
    rider::Rider,
  },
  traits::Blockable,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::MyUserInfo;
use app_108jobs_db_views_wallet::WalletView;

pub async fn get_my_user(
  local_user_view: LocalUserView,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<MyUserInfo>> {
  check_local_user_valid(&local_user_view)?;

  let person_id = local_user_view.person.id;
  let local_user_id = local_user_view.local_user.id;
  let pool = &mut context.pool();

  let (instance_blocks, person_blocks, keyword_blocks, discussion_languages, wallet, is_rider) =
    app_108jobs_db::try_join_with_pool!(pool => (
      |pool| InstanceActions::read_blocks_for_person(pool, person_id),
      |pool| PersonActions::read_blocks_for_person(pool, person_id),
      |pool| LocalUserKeywordBlock::read(pool, local_user_id),
      |pool| LocalUserLanguage::read(pool, local_user_id),
      |pool| WalletView::read_by_user(pool, local_user_id),
      |pool| Rider::is_verified_for_user(pool, local_user_id)
    ))?;

  Ok(Json(MyUserInfo {
    local_user_view: local_user_view.clone(),
    category_blocks: Vec::new(),
    instance_blocks,
    person_blocks,
    keyword_blocks,
    discussion_languages,
    wallet,
    is_rider,
  }))
}
