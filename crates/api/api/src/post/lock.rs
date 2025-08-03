use activitypub_federation::config::Data;
use actix_web::web::{Json};
use lemmy_api_utils::{
  build_response::build_post_response,
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData}
  ,
};
use lemmy_db_schema::{
  source::{
    mod_log::moderator::{ModLockPost, ModLockPostForm},
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::api::{LockPost, PostResponse};
use lemmy_utils::error::FastJobResult;

pub async fn lock_post(
  data: Json<LockPost>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PostResponse>> {
  // Update the post
  let post_id = data.post_id;
  let locked = data.locked;
  let post = Post::update(
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

  ActivityChannel::submit_activity(
    SendActivityData::LockPost(
      post,
      local_user_view.person.clone(),
      data.locked,
      data.reason.clone(),
    ),
    &context,
  )?;

  build_post_response(&context, local_user_view, post_id).await
}
