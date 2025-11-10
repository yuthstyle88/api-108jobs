use crate::context::FastJobContext;
use crate::utils::check_category_deleted_removed;
use lemmy_db_schema::{
  newtypes::TagId,
  source::{
    post::Post,
    post_tag::{PostTag, PostTagForm},
  },
};
use lemmy_db_views_category::CategoryView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use std::collections::HashSet;

pub async fn update_post_tags(
  context: &FastJobContext,
  post: &Post,
  category: &CategoryView,
  tags: &[TagId],
  local_user_view: &LocalUserView,
) -> FastJobResult<()> {
  let is_author = Post::is_post_creator(local_user_view.person.id, post.creator_id);

  if !is_author {
    check_category_deleted_removed(&category.category)?;
  }

  // validate tags
  let valid_tags: HashSet<TagId> = category.post_tags.0.iter().map(|t| t.id).collect();
  if !valid_tags.is_superset(&tags.iter().copied().collect()) {
    return Err(FastJobErrorType::TagNotIncategory.into());
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
