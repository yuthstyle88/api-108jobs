use crate::utils::check_community_deleted_removed;
use crate::context::FastJobContext;
use lemmy_db_schema::{
  newtypes::TagId,
  source::{
    post::Post,
    post_tag::{PostTag, PostTagForm},
  },
};
use lemmy_db_views_community::CommunityView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use std::collections::HashSet;

pub async fn update_post_tags(
  context: &FastJobContext,
  post: &Post,
  community: &CommunityView,
  tags: &[TagId],
  local_user_view: &LocalUserView,
) -> FastJobResult<()> {
  let is_author = Post::is_post_creator(local_user_view.person.id, post.creator_id);

  if !is_author {
    check_community_deleted_removed(&community.community)?;
  }

  // validate tags
  let valid_tags: HashSet<TagId> = community.post_tags.0.iter().map(|t| t.id).collect();
  if !valid_tags.is_superset(&tags.iter().copied().collect()) {
    return Err(FastJobErrorType::TagNotInCommunity.into());
  }

  let insert_tags = tags
    .iter()
    .map(|tag_id| PostTagForm {
      post_id: post.id,
      tag_id: *tag_id,
    })
    .collect::<Vec<PostTagForm>>();

  PostTag::set(&mut context.pool(), &insert_tags).await?;
  Ok(())
}
