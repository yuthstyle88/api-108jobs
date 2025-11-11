use actix_web::web::Data;
use actix_web::web::Json;
use lemmy_api_utils::utils::check_category_deleted_removed;
use lemmy_api_utils::{
  build_response::build_post_response,
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_db_schema::{
  source::{
    category::Category,
    local_user::LocalUser,
    mod_log::moderator::{ModRemovePost, ModRemovePostForm},
    post::{Post, PostUpdateForm},
    post_report::PostReport,
  },
  traits::{Crud, Reportable},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::api::{PostResponse, RemovePost};
use lemmy_utils::error::FastJobResult;

pub async fn remove_post(
  data: Json<RemovePost>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PostResponse>> {
  let post_id = data.post_id;

  // We cannot use PostView to avoid a database read here, as it doesn't return removed items
  // by default. So we would have to pass in `is_mod_or_admin`, but that is impossible without
  // knowing which category the post belongs to.
  let orig_post = Post::read(&mut context.pool(), post_id).await?;
  let category = Category::read(&mut context.pool(), orig_post.category_id).await?;

  check_category_deleted_removed(&category)?;

  LocalUser::is_higher_mod_or_admin_check(
    &mut context.pool(),
    orig_post.category_id,
    local_user_view.person.id,
    vec![orig_post.creator_id],
  )
  .await?;

  // Update the post
  let post_id = data.post_id;
  let removed = data.removed;
  let post = Post::update(
    &mut context.pool(),
    post_id,
    &PostUpdateForm {
      removed: Some(removed),
      ..Default::default()
    },
  )
  .await?;

  PostReport::resolve_all_for_object(&mut context.pool(), post_id, local_user_view.person.id)
    .await?;

  // Mod tables
  let form = ModRemovePostForm {
    mod_person_id: local_user_view.person.id,
    post_id: data.post_id,
    removed: Some(removed),
    reason: data.reason.clone(),
  };
  ModRemovePost::create(&mut context.pool(), &form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::RemovePost {
      post,
      moderator: local_user_view.person.clone(),
      reason: data.reason.clone(),
      removed: data.removed,
    },
    &context,
  )?;

  build_post_response(&context, local_user_view, post_id).await
}
