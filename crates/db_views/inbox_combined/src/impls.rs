use crate::{
  InboxCombinedView, InboxCombinedViewInternal, PersonPostMentionView, PersonProposalMentionView,
  ProposalReplyView,
};
use app_108jobs_core::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use app_108jobs_db::{
  aliases::{self},
  newtypes::{InstanceId, PaginationCursor, PersonId},
  schema::{
    inbox_combined, instance_actions, person, person_actions, person_post_mention,
    person_proposal_mention, post, proposal, proposal_reply,
  },
  source::combined::inbox::{inbox_combined_keys as key, InboxCombined},
  traits::{InternalToCombinedView, PaginationCursorBuilder},
  utils::{
    get_conn, limit_fetch, paginate,
    queries::{
      category_join, creator_category_actions_join, creator_home_instance_actions_join,
      creator_local_instance_actions_join, creator_local_user_admin_join, image_details_join,
      my_category_actions_join, my_instance_actions_person_join, my_local_user_admin_join,
      my_person_actions_join, my_post_actions_join, my_proposal_actions_join,
    },
    DbPool,
  },
  InboxDataType,
};
use diesel::{
  dsl::not, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;

impl InboxCombinedViewInternal {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: PersonId, local_instance_id: InstanceId) -> _ {
    let item_creator = person::id;
    let recipient_person = aliases::person1.field(person::id);

    let item_creator_join = person::table.on(
      proposal::creator_id.eq(item_creator).or(
        inbox_combined::person_post_mention_id
          .is_not_null()
          .and(post::creator_id.eq(item_creator)),
      ),
    );

    let recipient_join = aliases::person1.on(
      proposal_reply::recipient_id
        .eq(recipient_person)
        .or(person_proposal_mention::recipient_id.eq(recipient_person))
        .or(person_post_mention::recipient_id.eq(recipient_person)),
    );

    let comment_join = proposal::table.on(
      proposal_reply::comment_id
        .eq(proposal::id)
        .or(person_proposal_mention::comment_id.eq(proposal::id))
        // Filter out the deleted / removed
        .and(not(proposal::deleted))
        .and(not(proposal::removed)),
    );

    let post_join = post::table.on(
      person_post_mention::post_id
        .eq(post::id)
        .or(proposal::post_id.eq(post::id))
        // Filter out the deleted / removed
        .and(not(post::deleted))
        .and(not(post::removed)),
    );

    let my_category_actions_join: my_category_actions_join =
      my_category_actions_join(Some(my_person_id));
    let my_post_actions_join: my_post_actions_join = my_post_actions_join(Some(my_person_id));
    let my_proposal_actions_join: my_proposal_actions_join =
      my_proposal_actions_join(Some(my_person_id));
    let my_local_user_admin_join: my_local_user_admin_join =
      my_local_user_admin_join(Some(my_person_id));
    let my_instance_actions_person_join: my_instance_actions_person_join =
      my_instance_actions_person_join(Some(my_person_id));
    let my_person_actions_join: my_person_actions_join = my_person_actions_join(Some(my_person_id));
    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(local_instance_id);

    inbox_combined::table
      .left_join(proposal_reply::table)
      .left_join(person_proposal_mention::table)
      .left_join(person_post_mention::table)
      .left_join(comment_join)
      .left_join(post_join)
      .left_join(category_join())
      .inner_join(item_creator_join)
      .inner_join(recipient_join)
      .left_join(image_details_join())
      .left_join(creator_category_actions_join())
      .left_join(my_local_user_admin_join)
      .left_join(creator_local_user_admin_join())
      .left_join(my_category_actions_join)
      .left_join(my_instance_actions_person_join)
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_local_instance_actions_join)
      .left_join(my_post_actions_join)
      .left_join(my_person_actions_join)
      .left_join(my_proposal_actions_join)
  }

  /// Gets the number of unread mentions
  pub async fn get_unread_count(
    pool: &mut DbPool<'_>,
    my_person_id: PersonId,
    local_instance_id: InstanceId,
    show_bot_accounts: bool,
  ) -> FastJobResult<i64> {
    use diesel::dsl::count;
    let conn = &mut get_conn(pool).await?;

    let recipient_person = aliases::person1.field(person::id);

    let unread_filter = proposal_reply::read
      .eq(false)
      .or(person_proposal_mention::read.eq(false))
      .or(person_post_mention::read.eq(false));

    let mut query = Self::joins(my_person_id, local_instance_id)
      // Filter for your user
      .filter(recipient_person.eq(my_person_id))
      // Filter unreads
      .filter(unread_filter)
      // Don't count replies from blocked users
      .filter(person_actions::blocked_at.is_null())
      .filter(instance_actions::blocked_at.is_null())
      .select(count(inbox_combined::id))
      .into_boxed();

    // These filters need to be kept in sync with the filters in queries().list()
    if !show_bot_accounts {
      query = query.filter(not(person::bot_account));
    }

    query
      .first::<i64>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}

impl PaginationCursorBuilder for InboxCombinedView {
  type CursorData = InboxCombined;

  fn to_cursor(&self) -> PaginationCursor {
    let (prefix, id) = match &self {
      InboxCombinedView::ProposalReply(v) => ('R', v.proposal_reply.id.0),
      InboxCombinedView::ProposalMention(v) => ('C', v.person_proposal_mention.id.0),
      InboxCombinedView::PostMention(v) => ('P', v.person_post_mention.id.0),
    };
    PaginationCursor::new_single(prefix, id)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    let conn = &mut get_conn(pool).await?;
    let pids = cursor.prefixes_and_ids();
    let (prefix, id) = pids
      .as_slice()
      .first()
      .ok_or(FastJobErrorType::CouldntParsePaginationToken)?;

    let mut query = inbox_combined::table
      .select(Self::CursorData::as_select())
      .into_boxed();

    query = match prefix {
      'R' => query.filter(inbox_combined::proposal_reply_id.eq(id)),
      'C' => query.filter(inbox_combined::person_proposal_mention_id.eq(id)),
      'P' => query.filter(inbox_combined::person_post_mention_id.eq(id)),
      _ => return Err(FastJobErrorType::CouldntParsePaginationToken.into()),
    };
    let token = query.first(conn).await?;

    Ok(token)
  }
}

#[derive(Default)]
pub struct InboxCombinedQuery {
  pub type_: Option<InboxDataType>,
  pub unread_only: Option<bool>,
  pub show_bot_accounts: Option<bool>,
  pub cursor_data: Option<InboxCombined>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
  pub no_limit: Option<bool>,
}

impl InboxCombinedQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    my_person_id: PersonId,
    local_instance_id: InstanceId,
  ) -> FastJobResult<Vec<InboxCombinedView>> {
    let conn = &mut get_conn(pool).await?;

    let recipient_person = aliases::person1.field(person::id);

    let mut query = InboxCombinedViewInternal::joins(my_person_id, local_instance_id)
      .select(InboxCombinedViewInternal::as_select())
      .into_boxed();

    if !self.no_limit.unwrap_or_default() {
      let limit = limit_fetch(self.limit)?;
      query = query.limit(limit);
    }

    // Filters
    if self.unread_only.unwrap_or_default() {
      query = query
        // The recipient filter (IE only show replies to you)
        .filter(recipient_person.eq(my_person_id))
        .filter(
          proposal_reply::read
            .eq(false)
            .or(person_proposal_mention::read.eq(false))
            .or(person_post_mention::read.eq(false)),
        );
    } else {
      // A special case for private messages: show messages FROM you also.
      // Use a not-null checks to catch the others
      query = query.filter(
        inbox_combined::proposal_reply_id
          .is_not_null()
          .and(recipient_person.eq(my_person_id))
          .or(
            inbox_combined::person_proposal_mention_id
              .is_not_null()
              .and(recipient_person.eq(my_person_id)),
          )
          .or(
            inbox_combined::person_post_mention_id
              .is_not_null()
              .and(recipient_person.eq(my_person_id)),
          ),
      );
    }

    if !self.show_bot_accounts.unwrap_or_default() {
      query = query.filter(not(person::bot_account));
    };

    // Dont show replies from blocked users or instances
    query = query
      .filter(person_actions::blocked_at.is_null())
      .filter(instance_actions::blocked_at.is_null());

    if let Some(type_) = self.type_ {
      query = match type_ {
        InboxDataType::All => query,
        InboxDataType::ProposalReply => {
          query.filter(inbox_combined::proposal_reply_id.is_not_null())
        }
        InboxDataType::ProposalMention => {
          query.filter(inbox_combined::person_proposal_mention_id.is_not_null())
        }
        InboxDataType::PostMention => {
          query.filter(inbox_combined::person_post_mention_id.is_not_null())
        }
      }
    }

    // Sorting by published
    let paginated_query = paginate(
      query,
      SortDirection::Desc,
      self.cursor_data,
      None,
      self.page_back,
    )
    .then_order_by(key::published_at)
    // Tie breaker
    .then_order_by(key::id);

    let res = paginated_query
      .load::<InboxCombinedViewInternal>(conn)
      .await?;

    // Map the query results to the enum
    let out = res
      .into_iter()
      .filter_map(InternalToCombinedView::map_to_enum)
      .collect();

    Ok(out)
  }
}

impl InternalToCombinedView for InboxCombinedViewInternal {
  type CombinedView = InboxCombinedView;

  fn map_to_enum(self) -> Option<Self::CombinedView> {
    // Use for a short alias
    let v = self;

    if let (Some(proposal_reply), Some(proposal), Some(post), Some(category)) = (
      v.proposal_reply,
      v.proposal.clone(),
      v.post.clone(),
      v.category.clone(),
    ) {
      Some(InboxCombinedView::ProposalReply(ProposalReplyView {
        proposal_reply,
        proposal,
        recipient: v.item_recipient,
        post,
        category,
        creator: v.item_creator,
        category_actions: v.category_actions,
        proposal_actions: v.proposal_actions,
        person_actions: v.person_actions,
        instance_actions: v.instance_actions,
        creator_is_admin: v.item_creator_is_admin,
        post_tags: v.post_tags,
        can_mod: v.can_mod,
        creator_banned: v.creator_banned,
        creator_banned_from_category: v.creator_banned_from_category,
        creator_is_moderator: v.creator_is_moderator,
      }))
    } else if let (Some(person_proposal_mention), Some(proposal), Some(post), Some(category)) = (
      v.person_proposal_mention,
      v.proposal,
      v.post.clone(),
      v.category.clone(),
    ) {
      Some(InboxCombinedView::ProposalMention(
        PersonProposalMentionView {
          person_proposal_mention,
          proposal,
          recipient: v.item_recipient,
          post,
          category,
          creator: v.item_creator,
          category_actions: v.category_actions,
          proposal_actions: v.proposal_actions,
          person_actions: v.person_actions,
          instance_actions: v.instance_actions,
          creator_is_admin: v.item_creator_is_admin,
          can_mod: v.can_mod,
          creator_banned: v.creator_banned,
          creator_banned_from_category: v.creator_banned_from_category,
          creator_is_moderator: v.creator_is_moderator,
        },
      ))
    } else if let (Some(person_post_mention), Some(post), Some(category)) =
      (v.person_post_mention, v.post, v.category)
    {
      Some(InboxCombinedView::PostMention(PersonPostMentionView {
        person_post_mention,
        post,
        category,
        creator: v.item_creator,
        recipient: v.item_recipient,
        category_actions: v.category_actions,
        person_actions: v.person_actions,
        instance_actions: v.instance_actions,
        post_actions: v.post_actions,
        image_details: v.image_details,
        creator_is_admin: v.item_creator_is_admin,
        post_tags: v.post_tags,
        can_mod: v.can_mod,
        creator_banned: v.creator_banned,
        creator_banned_from_category: v.creator_banned_from_category,
        creator_is_moderator: v.creator_is_moderator,
      }))
    } else {
      None
    }
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {
  use crate::{impls::InboxCombinedQuery, InboxCombinedView, InboxCombinedViewInternal};
  use app_108jobs_core::error::FastJobResult;
  use app_108jobs_db::{
    assert_length,
    source::{
      category::{Category, CategoryInsertForm},
      instance::Instance,
      person::{Person, PersonActions, PersonBlockForm, PersonInsertForm, PersonUpdateForm},
      person_post_mention::{PersonPostMention, PersonPostMentionInsertForm},
      person_proposal_mention::{PersonProposalMention, PersonProposalMentionInsertForm},
      post::{Post, PostInsertForm},
      proposal::{Proposal, ProposalInsertForm},
      proposal_reply::{ProposalReply, ProposalReplyInsertForm, ProposalReplyUpdateForm},
    },
    traits::{Blockable, Crud},
    utils::{build_db_pool_for_tests, DbPool},
    InboxDataType,
  };
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  struct Data {
    instance: Instance,
    timmy: Person,
    sara: Person,
    jessica: Person,
    timmy_post: Post,
    jessica_post: Post,
    timmy_comment: Proposal,
    sara_comment: Proposal,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> FastJobResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let (timmy_form, _) =
      PersonInsertForm::test_form_with_wallet(pool, instance.id, "timmy_pcv").await?;
    let timmy = Person::create(pool, &timmy_form).await?;

    let (sara_form, _) =
      PersonInsertForm::test_form_with_wallet(pool, instance.id, "sara_pcv").await?;
    let sara = Person::create(pool, &sara_form).await?;

    let (jessica_form, _) =
      PersonInsertForm::test_form_with_wallet(pool, instance.id, "jessica_mrv").await?;
    let jessica = Person::create(pool, &jessica_form).await?;

    let category_form = CategoryInsertForm::new(
      instance.id,
      "test category pcv".to_string(),
      "nada".to_owned(),
    );
    let category = Category::create(pool, &category_form).await?;

    let timmy_post_form = PostInsertForm {
      category_id: Some(category.id),
      ..PostInsertForm::new("timmy post prv".into(), timmy.id)
    };
    let timmy_post = Post::create(pool, &timmy_post_form).await?;

    let jessica_post_form = PostInsertForm {
      category_id: Some(category.id),
      ..PostInsertForm::new("jessica post prv".into(), jessica.id)
    };
    let jessica_post = Post::create(pool, &jessica_post_form).await?;

    let timmy_comment_form =
      ProposalInsertForm::new(timmy.id, timmy_post.id, "timmy proposal prv".into());
    let timmy_comment = Proposal::create(pool, &timmy_comment_form).await?;

    let sara_comment_form =
      ProposalInsertForm::new(sara.id, timmy_post.id, "sara proposal prv".into());
    let sara_comment = Proposal::create(pool, &sara_comment_form).await?;

    Ok(Data {
      instance,
      timmy,
      sara,
      jessica,
      timmy_post,
      jessica_post,
      timmy_comment,
      sara_comment,
    })
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> FastJobResult<()> {
    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn replies() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Sara replied to timmys proposal, but let create the row now
    let form = ProposalReplyInsertForm {
      recipient_id: data.timmy.id,
      comment_id: data.sara_comment.id,
      read: None,
    };
    let reply = ProposalReply::create(pool, &form).await?;

    let timmy_unread_replies =
      InboxCombinedViewInternal::get_unread_count(pool, data.timmy.id, data.instance.id, true)
        .await?;
    assert_eq!(1, timmy_unread_replies);

    let timmy_inbox = InboxCombinedQuery::default()
      .list(pool, data.timmy.id, data.instance.id)
      .await?;
    assert_length!(1, timmy_inbox);

    if let InboxCombinedView::ProposalReply(v) = &timmy_inbox[0] {
      assert_eq!(data.sara_comment.id, v.proposal_reply.comment_id);
      assert_eq!(data.sara_comment.id, v.proposal.id);
      assert_eq!(data.timmy_post.id, v.post.id);
      assert_eq!(data.sara.id, v.creator.id);
      assert_eq!(data.timmy.id, v.recipient.id);
    } else {
      panic!("wrong type");
    }

    // Mark it as read
    let form = ProposalReplyUpdateForm { read: Some(true) };
    ProposalReply::update(pool, reply.id, &form).await?;

    let timmy_unread_replies =
      InboxCombinedViewInternal::get_unread_count(pool, data.timmy.id, data.instance.id, true)
        .await?;
    assert_eq!(0, timmy_unread_replies);

    let timmy_inbox_unread = InboxCombinedQuery {
      unread_only: Some(true),
      ..Default::default()
    }
    .list(pool, data.timmy.id, data.instance.id)
    .await?;
    assert_length!(0, timmy_inbox_unread);

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn mentions() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Timmy mentions sara in a proposal
    let timmy_mention_sara_comment_form = PersonProposalMentionInsertForm {
      recipient_id: data.sara.id,
      comment_id: data.timmy_comment.id,
      read: None,
    };
    PersonProposalMention::create(pool, &timmy_mention_sara_comment_form).await?;

    // Jessica mentions sara in a post
    let jessica_mention_sara_post_form = PersonPostMentionInsertForm {
      recipient_id: data.sara.id,
      post_id: data.jessica_post.id,
      read: None,
    };
    PersonPostMention::create(pool, &jessica_mention_sara_post_form).await?;

    // Test to make sure counts and blocks work correctly
    let sara_unread_mentions =
      InboxCombinedViewInternal::get_unread_count(pool, data.sara.id, data.instance.id, true)
        .await?;
    assert_eq!(2, sara_unread_mentions);

    let sara_inbox = InboxCombinedQuery::default()
      .list(pool, data.sara.id, data.instance.id)
      .await?;
    assert_length!(2, sara_inbox);

    if let InboxCombinedView::PostMention(v) = &sara_inbox[0] {
      assert_eq!(data.jessica_post.id, v.person_post_mention.post_id);
      assert_eq!(data.jessica_post.id, v.post.id);
      assert_eq!(data.jessica.id, v.creator.id);
      assert_eq!(data.sara.id, v.recipient.id);
    } else {
      panic!("wrong type");
    }

    if let InboxCombinedView::ProposalMention(v) = &sara_inbox[1] {
      assert_eq!(data.timmy_comment.id, v.person_proposal_mention.comment_id);
      assert_eq!(data.timmy_comment.id, v.proposal.id);
      assert_eq!(data.timmy_post.id, v.post.id);
      assert_eq!(data.timmy.id, v.creator.id);
      assert_eq!(data.sara.id, v.recipient.id);
    } else {
      panic!("wrong type");
    }

    // Sara blocks timmy, and makes sure these counts are now empty
    let sara_blocks_timmy_form = PersonBlockForm::new(data.sara.id, data.timmy.id);
    PersonActions::block(pool, &sara_blocks_timmy_form).await?;

    let sara_unread_mentions_after_block =
      InboxCombinedViewInternal::get_unread_count(pool, data.sara.id, data.instance.id, true)
        .await?;
    assert_eq!(1, sara_unread_mentions_after_block);

    let sara_inbox_after_block = InboxCombinedQuery::default()
      .list(pool, data.sara.id, data.instance.id)
      .await?;
    assert_length!(1, sara_inbox_after_block);

    // Make sure the proposal mention which timmy made is the hidden one
    assert!(matches!(
      sara_inbox_after_block[0],
      InboxCombinedView::PostMention(_)
    ));

    // Unblock user so we can reuse the same person
    PersonActions::unblock(pool, &sara_blocks_timmy_form).await?;

    // Test the type filter
    let sara_inbox_post_mentions_only = InboxCombinedQuery {
      type_: Some(InboxDataType::PostMention),
      ..Default::default()
    }
    .list(pool, data.sara.id, data.instance.id)
    .await?;
    assert_length!(1, sara_inbox_post_mentions_only);

    assert!(matches!(
      sara_inbox_post_mentions_only[0],
      InboxCombinedView::PostMention(_)
    ));

    // Turn Jessica into a bot account
    let person_update_form = PersonUpdateForm {
      bot_account: Some(true),
      ..Default::default()
    };
    Person::update(pool, data.jessica.id, &person_update_form).await?;

    // Make sure sara hides bots
    let sara_unread_mentions_after_hide_bots =
      InboxCombinedViewInternal::get_unread_count(pool, data.sara.id, data.instance.id, false)
        .await?;
    assert_eq!(1, sara_unread_mentions_after_hide_bots);

    let sara_inbox_after_hide_bots = InboxCombinedQuery::default()
      .list(pool, data.sara.id, data.instance.id)
      .await?;
    assert_length!(1, sara_inbox_after_hide_bots);

    // Make sure the post mention which jessica made is the hidden one
    assert!(matches!(
      sara_inbox_after_hide_bots[0],
      InboxCombinedView::ProposalMention(_)
    ));

    // Mark them all as read
    PersonPostMention::mark_all_as_read(pool, data.sara.id).await?;
    PersonProposalMention::mark_all_as_read(pool, data.sara.id).await?;

    // Make sure none come back
    let sara_unread_mentions =
      InboxCombinedViewInternal::get_unread_count(pool, data.sara.id, data.instance.id, false)
        .await?;
    assert_eq!(0, sara_unread_mentions);

    let sara_inbox_unread = InboxCombinedQuery {
      unread_only: Some(true),
      ..Default::default()
    }
    .list(pool, data.sara.id, data.instance.id)
    .await?;
    assert_length!(0, sara_inbox_unread);

    cleanup(data, pool).await?;

    Ok(())
  }
}
