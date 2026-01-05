use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::utils::check_category_deleted_removed;
use app_108jobs_api_utils::{
  build_response::build_post_response,
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::is_admin,
};
use app_108jobs_db_schema::{
  source::{
    category::Category,
    mod_log::moderator::{ModFeaturePost, ModFeaturePostForm},
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
  PostFeatureType,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::api::{FeaturePost, PostResponse};
use app_108jobs_utils::error::FastJobResult;

pub async fn feature_post(
  data: Json<FeaturePost>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PostResponse>> {
  let post_id = data.post_id;
  let orig_post = Post::read(&mut context.pool(), post_id).await?;

  let category = Category::read(&mut context.pool(), orig_post.category_id).await?;
  check_category_deleted_removed(&category)?;

  if data.feature_type == PostFeatureType::Local {
    is_admin(&local_user_view)?;
  }

  // Update the post
  let post_id = data.post_id;
  let new_post: PostUpdateForm = if data.feature_type == PostFeatureType::Category {
    PostUpdateForm {
      featured_category: Some(data.featured),
      ..Default::default()
    }
  } else {
    PostUpdateForm {
      featured_local: Some(data.featured),
      ..Default::default()
    }
  };
  let post = Post::update(&mut context.pool(), post_id, &new_post).await?;

  // Mod tables
  let form = ModFeaturePostForm {
    mod_person_id: local_user_view.person.id,
    post_id: data.post_id,
    featured: Some(data.featured),
    is_featured_category: Some(data.feature_type == PostFeatureType::Category),
  };

  ModFeaturePost::create(&mut context.pool(), &form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::FeaturePost(post, local_user_view.person.clone(), data.featured),
    &context,
  )?;

  build_post_response(&context, local_user_view, post_id).await
}
