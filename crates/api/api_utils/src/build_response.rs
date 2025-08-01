use crate::context::FastJobContext;
use actix_web::web::Json;
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, InstanceId, PostId},
  source::{
    actor_language::CommunityLanguage,
    comment::Comment,
    community::Community,
    person::Person,
    person_comment_mention::{PersonCommentMention, PersonCommentMentionInsertForm},
    person_post_mention::{PersonPostMention, PersonPostMentionInsertForm},
    post::Post,
  },
  traits::Crud,
};
use lemmy_db_views_comment::{api::CommentResponse, CommentView};
use lemmy_db_views_community::api::ListCommunitiesTreeResponse;
use lemmy_db_views_community::{api::CommunityResponse, CommunityNodeView, CommunityView};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::{api::PostResponse, PostView};
use lemmy_utils::{error::FastJobResult, utils::mention::scrape_text_for_mentions};
use std::collections::{HashMap, HashSet};
use url::Url;

pub async fn build_comment_response(
  context: &FastJobContext,
  comment_id: CommentId,
  local_user_view: Option<LocalUserView>,
  local_instance_id: InstanceId,
) -> FastJobResult<CommentResponse> {
  let local_user = local_user_view.map(|l| l.local_user);
  let comment_view = CommentView::read(
    &mut context.pool(),
    comment_id,
    local_user.as_ref(),
    local_instance_id,
  )
  .await?;
  Ok(CommentResponse { comment_view })
}

pub async fn build_community_response(
  context: &FastJobContext,
  local_user_view: LocalUserView,
  community_id: CommunityId,
) -> FastJobResult<Json<CommunityResponse>> {
  let local_user = local_user_view.local_user;
  let community_view =
    CommunityView::read(&mut context.pool(), community_id, Some(&local_user)).await?;
  let discussion_languages = CommunityLanguage::read(&mut context.pool(), community_id).await?;

  Ok(Json(CommunityResponse {
    community_view,
    discussion_languages,
  }))
}

pub async fn build_post_response(
  context: &FastJobContext,
  local_user_view: LocalUserView,
  post_id: PostId,
) -> FastJobResult<Json<PostResponse>> {
  let local_user = local_user_view.local_user;
  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user),
    local_user_view.person.instance_id,
  )
  .await?;
  Ok(Json(PostResponse { post_view }))
}

pub fn build_community_tree(
  flat_list: Vec<Community>,
) -> FastJobResult<Json<ListCommunitiesTreeResponse>> {
  let mut node_map: HashMap<String, CommunityNodeView> = HashMap::new();
  let mut all_children: HashSet<String> = HashSet::new();

  // First pass: build all nodes
  for community in &flat_list {
    let path_str = community.path.0.to_string();
    node_map.insert(
      path_str.clone(),
      CommunityNodeView {
        community: community.clone(),
        children: Vec::new(),
      },
    );
  }

  // Second pass: build tree
  for community in &flat_list {
    let path_str = community.path.0.to_string();
    let segments: Vec<&str> = path_str.split('.').collect();

    if segments.len() > 1 {
      let parent_path = segments[..segments.len() - 1].join(".");
      if let Some(current_node) = node_map.get(&path_str).cloned() {
        if let Some(parent_node) = node_map.get_mut(&parent_path) {
          parent_node.children.push(current_node);
          all_children.insert(path_str);
        }
      }
    }
  }

  // Collect root nodes (not children of anyone else)
  let roots: Vec<CommunityNodeView> = node_map
    .into_iter()
    .filter_map(|(path, node)| {
      if !all_children.contains(&path) {
        Some(node)
      } else {
        None
      }
    })
    .collect();

  Ok(Json(ListCommunitiesTreeResponse {
    communities: roots.clone(),
    count: roots.len() as i32,
  }))
}

/// Scans the post/comment content for mentions, then sends notifications via db and multilang
/// to mentioned users and parent creator.
pub async fn send_local_notifs(
  post: &Post,
  comment_opt: Option<&Comment>,
  person: &Person,
  context: &FastJobContext,
) -> FastJobResult<()> {
  send_local_mentions(post, comment_opt, person, context).await?;

  Ok(())
}

async fn send_local_mentions(
  post: &Post,
  comment_opt: Option<&Comment>,
  person: &Person,
  context: &FastJobContext,
) -> FastJobResult<()> {
  let content = if let Some(comment) = comment_opt {
    &comment.content
  } else {
    &post.body.clone().unwrap_or_default()
  };
  let mentions = scrape_text_for_mentions(content)
    .into_iter()
    .filter(|m| m.is_local(&context.settings().hostname) && m.name.ne(&person.name));
  for mention in mentions {
    // Ignore error if user is remote
    let Ok(user_view) = LocalUserView::read_from_name(&mut context.pool(), &mention.name).await
    else {
      continue;
    };

    let _ = insert_post_or_comment_mention(&user_view, post, comment_opt, context).await?;
  }
  Ok(())
}

/// Make the correct reply form depending on whether its a post or comment mention
async fn insert_post_or_comment_mention(
  mention_user_view: &LocalUserView,
  post: &Post,
  comment_opt: Option<&Comment>,
  context: &FastJobContext,
) -> FastJobResult<(Url, String)> {
  if let Some(comment) = &comment_opt {
    let person_comment_mention_form = PersonCommentMentionInsertForm {
      recipient_id: mention_user_view.person.id,
      comment_id: comment.id,
      read: None,
    };

    // Allow this to fail softly, since comment edits might re-update or replace it
    // Let the uniqueness handle this fail
    PersonCommentMention::create(&mut context.pool(), &person_comment_mention_form)
      .await
      .ok();
    Ok((
      comment.local_url(context.settings())?,
      comment.content.clone(),
    ))
  } else {
    let person_post_mention_form = PersonPostMentionInsertForm {
      recipient_id: mention_user_view.person.id,
      post_id: post.id,
      read: None,
    };

    // Allow this to fail softly, since edits might re-update or replace it
    PersonPostMention::create(&mut context.pool(), &person_post_mention_form)
      .await
      .ok();
    Ok((
      post.local_url(context.settings())?,
      post.body.clone().unwrap_or_default(),
    ))
  }
}
