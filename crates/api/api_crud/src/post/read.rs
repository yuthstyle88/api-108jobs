use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::{
  context::FastJobContext,
  utils::{check_private_instance, update_read_comments},
};
use app_108jobs_db_schema::{
  source::{
    comment::Comment,
    post::{Post, PostActions, PostReadForm},
  },
  traits::{Crud, Readable}
  ,
};
use app_108jobs_db_views_category::CategoryView;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::{
  api::{GetPost, GetPostResponse},
  PostView,
};
use app_108jobs_db_views_post::logistics::{self, LogisticsViewer};
use app_108jobs_db_views_search_combined::impls::SearchCombinedQuery;
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};

pub async fn get_post(
  data: Query<GetPost>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<GetPostResponse>> {
  let site_view = context.site_config().get().await?.site_view;
  let local_site = site_view.local_site;
  let local_instance_id = site_view.site.instance_id;

  check_private_instance(&local_user_view, &local_site)?;

  let person_id = local_user_view.as_ref().map(|u| u.person.id);
  let local_user = local_user_view.as_ref().map(|l| l.local_user.clone());

  // I'd prefer fetching the post_view by a comment join, but it adds a lot of boilerplate
  let post_id = if let Some(id) = data.id {
    id
  } else if let Some(comment_id) = data.comment_id {
    Comment::read(&mut context.pool(), comment_id)
      .await?
      .post_id
  } else {
    Err(FastJobErrorType::NotFound)?
  };

  // Check to see if the person is a mod or admin, to show deleted / removed
  let category_id = Post::read_xx(&mut context.pool(), post_id)
    .await?
    .category_id;

  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    local_user.as_ref(),
    local_instance_id,
  )
  .await?;

  let post_id = post_view.post.id;
  if let Some(person_id) = person_id {
    let read_form = PostReadForm::new(post_id, person_id);
    PostActions::mark_as_read(&mut context.pool(), &read_form).await?;

    update_read_comments(
      person_id,
      post_id,
      post_view.post.comments,
      &mut context.pool(),
    )
    .await?;
  }

  // Necessary for the sidebar subscribed
  let category_view = if let Some(cid) = category_id {
    Some(
      CategoryView::read(
        &mut context.pool(),
        cid,
        local_user.as_ref(),
      )
      .await?,
    )
  } else {
    None
  };

  // Fetch the cross_posts
  let cross_posts = if let Some(url) = &post_view.post.url {
    SearchCombinedQuery {
      search_term: Some(url.inner().as_str().into()),
      ..Default::default()
    }
    .list(&mut context.pool(), &local_user_view, &site_view.site)
    .await?
    .iter()
    // Filter map to collect posts
    .filter_map(|f| f.to_post_view())
    // Don't return this post as one of the cross_posts
    .filter(|x| x.post.id != post_id)
    .cloned()
    .collect::<Vec<PostView>>()
  } else {
    Vec::new()
  };

  // Return the jwt
  // Compute viewer and load logistics view if applicable
  let (viewer, is_admin) = if let Some(lu) = local_user.as_ref() {
    if lu.admin {
      (LogisticsViewer::Admin, true)
    } else if let Some(p) = person_id {
      if p == post_view.creator.id {
        (LogisticsViewer::Employer(post_view.creator.id), false)
      } else {
        (LogisticsViewer::Public, false)
      }
    } else {
      (LogisticsViewer::Public, false)
    }
  } else {
    (LogisticsViewer::Public, false)
  };

  let logistics = logistics::load_post_logistics(
    &mut context.pool(),
    post_id,
    post_view.post.post_kind,
    post_view.creator.id,
    viewer,
    is_admin,
  )
  .await?;

  Ok(Json(GetPostResponse { post_view, category_view, cross_posts, logistics }))
}
