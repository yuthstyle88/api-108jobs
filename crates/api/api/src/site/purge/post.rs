use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::{
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{is_admin, purge_post_images},
};
use app_108jobs_db_schema::{
  source::{
    local_user::LocalUser,
    mod_log::admin::{AdminPurgePost, AdminPurgePostForm},
    post::Post,
  },
  traits::Crud,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::api::PurgePost;
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_utils::error::FastJobResult;

pub async fn purge_post(
  data: Json<PurgePost>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  // Only let admin purge an item
  is_admin(&local_user_view)?;

  // Read the post to get the category_id
  let post = Post::read(&mut context.pool(), data.post_id).await?;

  // Also check that you're a higher admin
  LocalUser::is_higher_admin_check(
    &mut context.pool(),
    local_user_view.person.id,
    vec![post.creator_id],
  )
  .await?;

  purge_post_images(post.url.clone(), post.thumbnail_url.clone(), &context).await;

  Post::delete(&mut context.pool(), data.post_id).await?;

  // Mod tables
  let form = AdminPurgePostForm {
    admin_person_id: local_user_view.person.id,
    reason: data.reason.clone(),
    category_id: post.category_id,
  };
  AdminPurgePost::create(&mut context.pool(), &form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::RemovePost {
      post,
      moderator: local_user_view.person.clone(),
      reason: data.reason.clone(),
      removed: true,
    },
    &context,
  )?;

  Ok(Json(SuccessResponse::default()))
}
