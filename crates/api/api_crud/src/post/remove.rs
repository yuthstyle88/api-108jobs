use actix_web::web::{Data, Json};
use app_108jobs_api_utils::{
  build_response::build_post_response,
  context::FastJobContext,
  utils::check_category_deleted_removed,
};
use app_108jobs_core::error::{FastJobErrorType, FastJobResult};
use app_108jobs_db_schema::{
  source::{
    category::Category,
    local_user::LocalUser,
    mod_log::moderator::{ModRemovePost, ModRemovePostForm},
    post::{Post, PostUpdateForm},
    post_report::PostReport,
  },
  traits::{Crud, Reportable},
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::api::{PostResponse, RemovePost};

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

  // For posts with a category, check category permissions
  if let Some(category_id) = orig_post.category_id {
    let category = Category::read(&mut context.pool(), category_id).await?;
    check_category_deleted_removed(&category)?;

    LocalUser::is_higher_mod_or_admin_check(
      &mut context.pool(),
      category_id,
      local_user_view.person.id,
      vec![orig_post.creator_id],
    )
    .await?;
  } else {
    // For delivery posts without a category, only admins can remove
    if !local_user_view.local_user.admin {
      return Err(FastJobErrorType::NoPostEditAllowed.into());
    }
  }

  // Update the post
  let post_id = data.post_id;
  let removed = data.removed;
  Post::update(
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

  build_post_response(&context, local_user_view, post_id).await
}
