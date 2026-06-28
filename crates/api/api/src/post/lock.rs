use actix_web::web::{Data, Json};
use app_108jobs_api_utils::{build_response::build_post_response, context::FastJobContext};
use app_108jobs_core::error::FastJobResult;
use app_108jobs_db_schema::{
  source::{
    mod_log::moderator::{ModLockPost, ModLockPostForm},
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::api::{LockPost, PostResponse};

pub async fn lock_post(
  data: Json<LockPost>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PostResponse>> {
  // Update the post
  let post_id = data.post_id;
  let locked = data.locked;
  Post::update(
    &mut context.pool(),
    post_id,
    &PostUpdateForm {
      locked: Some(locked),
      ..Default::default()
    },
  )
  .await?;

  // Mod tables
  let form = ModLockPostForm {
    mod_person_id: local_user_view.person.id,
    post_id: data.post_id,
    locked: Some(locked),
    reason: data.reason.clone(),
  };
  ModLockPost::create(&mut context.pool(), &form).await?;

  build_post_response(&context, local_user_view, post_id).await
}
