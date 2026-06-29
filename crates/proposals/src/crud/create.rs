use actix_web::web::{Data, Json};
use app_108jobs_api_utils::{
  build_response::{build_proposal_response, send_local_notifs},
  context::FastJobContext,
  utils::{
    check_post_deleted_or_removed, get_url_blocklist, process_markdown, slur_regex,
    update_read_proposals,
  },
};
use app_108jobs_core::{
  error::{FastJobError, FastJobErrorType, FastJobResult},
  utils::validation::is_valid_body_field,
};
use app_108jobs_db::{
  enums::PostKind,
  impls::actor_language::{validate_post_language, UNDETERMINED_ID},
  newtypes::{PersonId, PostId},
  schema::proposal::{creator_id, deleted, dsl::proposal, post_id, removed},
  source::proposal::{Proposal, ProposalActions, ProposalInsertForm, ProposalLikeForm},
  traits::{Crud, Likeable},
  utils::DbPool,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::PostView;
use app_108jobs_db_views_proposal::api::{CreateComment, CreateCommentRequest, ProposalResponse};

pub async fn create_comment(
  data: Json<CreateCommentRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ProposalResponse>> {
  let request_data = data.into_inner();

  let data: CreateComment = request_data.try_into()?;
  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  let content = process_markdown(&data.content, &slur_regex, &url_blocklist, &context).await?;
  is_valid_body_field(&content, false)?;

  let local_instance_id = local_user_view.person.instance_id;

  // Read the full post view in order to get the comments count.
  let post_view = PostView::read(
    &mut context.pool(),
    data.post_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  let post = post_view.post;

  // Handle delivery/ridetaxi posts which have no category
  let language_id = if let Some(category) = post_view.category {
    // Normal post with a category - validate language
    validate_post_language(
      &mut context.pool(),
      data.language_id,
      category.id,
      local_user_view.local_user.id,
    )
    .await?
  } else if matches!(post.post_kind, PostKind::Delivery | PostKind::RideTaxi) {
    // Delivery or RideTaxi post without a category - use provided language or UNDETERMINED_ID
    data.language_id.unwrap_or(UNDETERMINED_ID)
  } else {
    // Non-delivery/ridetaxi post without a category should not happen
    return Err(FastJobErrorType::NotFound)?;
  };

  check_post_deleted_or_removed(&post)?;

  // Check if post is locked, no new comments
  if post.locked {
    Err(FastJobErrorType::Locked)?
  }

  // Check if user is trying to comment on their own post
  if post.creator_id == local_user_view.person.id {
    return Err(FastJobErrorType::CannotProposeOnOwnPost)?;
  }

  // Check if user has already commented on this post
  check_user_already_commented(
    &mut context.pool(),
    local_user_view.person.id,
    data.post_id.clone(),
  )
  .await?;

  let proposal_form = ProposalInsertForm {
    language_id: Some(language_id),
    ..ProposalInsertForm::new(local_user_view.person.id, data.post_id, content.clone())
  };

  // Create the proposal
  let inserted_proposal = Proposal::create(&mut context.pool(), &proposal_form).await?;

  send_local_notifs(
    &post,
    Some(&inserted_proposal),
    &local_user_view.person,
    &context,
  )
  .await?;

  // You like your own proposal by default
  let like_form = ProposalLikeForm::new(local_user_view.person.id, inserted_proposal.id, 1);

  ProposalActions::like(&mut context.pool(), &like_form).await?;

  // Update the read proposals, so your own new proposal doesn't appear as a +1 unread
  update_read_proposals(
    local_user_view.person.id,
    data.post_id.clone(),
    post.proposals + 1,
    &mut context.pool(),
  )
  .await?;

  // If we're responding to a comment where we're the recipient,
  // (ie we're the grandparent, or the recipient of the parent comment_reply),
  // then mark the parent as read.
  // Then we don't have to do it manually after we respond to a comment.

  Ok(Json(
    build_proposal_response(
      &context,
      inserted_proposal.id,
      Some(local_user_view),
      local_instance_id,
    )
    .await?,
  ))
}

async fn check_user_already_commented(
  pool: &mut DbPool<'_>,
  person_id: PersonId,
  current_post_id: PostId,
) -> FastJobResult<()> {
  use app_108jobs_db::utils::get_conn;
  use diesel::prelude::*;
  use diesel_async::RunQueryDsl;

  let conn = &mut get_conn(pool).await?;

  let existing_comment = proposal
    .filter(creator_id.eq(person_id))
    .filter(post_id.eq(current_post_id))
    .filter(deleted.eq(false))
    .filter(removed.eq(false))
    .first::<Proposal>(conn)
    .await
    .optional()
    .map_err(|_| FastJobError::from(FastJobErrorType::DatabaseError))?;

  if existing_comment.is_some() {
    return Err(FastJobErrorType::AlreadyProposed)?;
  }

  Ok(())
}
