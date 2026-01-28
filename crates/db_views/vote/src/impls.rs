use crate::VoteView;
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use app_108jobs_db_schema::{
  aliases::creator_category_actions,
  newtypes::{CommentId, PaginationCursor, PersonId, PostId},
  source::{comment::CommentActions, post::PostActions},
  utils::{get_conn, limit_fetch, paginate, DbPool},
};
use app_108jobs_db_schema_file::schema::{
    comment,
    comment_actions,
    category_actions,
    person,
    post,
    post_actions,
};
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl VoteView {
  pub fn to_post_actions_cursor(&self) -> PaginationCursor {
    // This needs a person and post
    let prefixes_and_ids = [('P', self.creator.id.0)];

    PaginationCursor::new(&prefixes_and_ids)
  }

  // TODO move this to the postactions impl soon.
  pub async fn from_post_actions_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<PostActions> {
    let pids = cursor.prefixes_and_ids();
    let (_, person_id) = pids
      .as_slice()
      .first()
      .ok_or(FastJobErrorType::CouldntParsePaginationToken)?;
    let (_, post_id) = pids
      .get(1)
      .ok_or(FastJobErrorType::CouldntParsePaginationToken)?;

    PostActions::read(pool, PostId(*post_id), PersonId(*person_id)).await
  }

  pub async fn list_for_post(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    cursor_data: Option<PostActions>,
    page_back: Option<bool>,
    limit: Option<i64>,
  ) -> FastJobResult<Vec<Self>> {
    use app_108jobs_db_schema::source::post::post_actions_keys as key;

    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(limit)?;

    let creator_category_actions_join = creator_category_actions.on(
      creator_category_actions
        .field(category_actions::category_id)
        .nullable()
        .eq(post::category_id)
        .and(
          creator_category_actions
            .field(category_actions::person_id)
            .eq(post_actions::person_id),
        ),
    );

    let query = post_actions::table
      .inner_join(person::table)
      .inner_join(post::table)
      .left_join(creator_category_actions_join)
      .filter(post_actions::post_id.eq(post_id))
      .filter(post_actions::like_score.is_not_null())
      .select((
        person::all_columns,
        creator_category_actions
          .field(category_actions::received_ban_at)
          .nullable()
          .is_not_null(),
        post_actions::like_score.assume_not_null(),
      ))
      .limit(limit)
      .into_boxed();

    // Sorting by like score
    let paginated_query = paginate(query, SortDirection::Asc, cursor_data, None, page_back)
      .then_order_by(key::like_score)
      // Tie breaker
      .then_order_by(key::liked_at);

    paginated_query
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub fn to_comment_actions_cursor(&self) -> PaginationCursor {
    // This needs a person and comment
    let prefixes_and_ids = [('P', self.creator.id.0)];

    PaginationCursor::new(&prefixes_and_ids)
  }

  pub async fn from_comment_actions_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<CommentActions> {
    let pids = cursor.prefixes_and_ids();
    let (_, person_id) = pids
      .as_slice()
      .first()
      .ok_or(FastJobErrorType::CouldntParsePaginationToken)?;
    let (_, comment_id) = pids
      .get(1)
      .ok_or(FastJobErrorType::CouldntParsePaginationToken)?;

    CommentActions::read(pool, CommentId(*comment_id), PersonId(*person_id)).await
  }

  pub async fn list_for_comment(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
    cursor_data: Option<CommentActions>,
    page_back: Option<bool>,
    limit: Option<i64>,
  ) -> FastJobResult<Vec<Self>> {
    use app_108jobs_db_schema::source::comment::comment_actions_keys as key;
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(limit)?;

    let creator_category_actions_join = creator_category_actions.on(
      creator_category_actions
        .field(category_actions::category_id)
        .nullable()
        .eq(post::category_id)
        .and(
          creator_category_actions
            .field(category_actions::person_id)
            .eq(comment_actions::person_id),
        ),
    );

    let query = comment_actions::table
      .inner_join(person::table)
      .inner_join(comment::table.inner_join(post::table))
      .left_join(creator_category_actions_join)
      .filter(comment_actions::comment_id.eq(comment_id))
      .filter(comment_actions::like_score.is_not_null())
      .select((
        person::all_columns,
        creator_category_actions
          .field(category_actions::received_ban_at)
          .nullable()
          .is_not_null(),
        comment_actions::like_score.assume_not_null(),
      ))
      .limit(limit)
      .into_boxed();

    // Sorting by like score
    let paginated_query = paginate(query, SortDirection::Asc, cursor_data, None, page_back)
      .then_order_by(key::like_score)
      // Tie breaker
      .then_order_by(key::liked_at);

    paginated_query
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}

#[cfg(test)]
mod tests {
  use crate::VoteView;
  use app_108jobs_db_schema::{
    source::{
        comment::{Comment, CommentActions, CommentInsertForm, CommentLikeForm},
        category::{Category, CategoryInsertForm},
        instance::Instance,
        person::{Person, PersonInsertForm},
        post::{Post, PostActions, PostInsertForm, PostLikeForm},
    },
    traits::{Crud, Likeable},
    utils::build_db_pool_for_tests,
  };
  use app_108jobs_utils::error::FastJobResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;
  use app_108jobs_db_schema::newtypes::DbUrl;

  #[tokio::test]
  #[serial]
  async fn post_and_comment_vote_views() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "timmy_vv");

    let inserted_timmy = Person::create(pool, &new_person).await?;

    let new_person_2 = PersonInsertForm::test_form(inserted_instance.id, "sara_vv");

    let inserted_sara = Person::create(pool, &new_person_2).await?;

    let new_category = CategoryInsertForm::new(
      inserted_instance.id,
      "test category vv".to_string(),
      "nada".to_owned(),
    );
    let inserted_category = Category::create(pool, &new_category).await?;

    let new_post = PostInsertForm::new(
      "A test post vv".into(),
      inserted_timmy.id,
      inserted_category.id,
    );
    let inserted_post = Post::create(pool, &new_post).await?;

    let comment_form = CommentInsertForm::new(
      inserted_timmy.id,
      inserted_post.id,
      "A test comment vv".into(),
    );
    let inserted_comment = Comment::create(pool, &comment_form,).await?;

    // Timmy upvotes his own post
    let timmy_post_vote_form = PostLikeForm::new(inserted_post.id, inserted_timmy.id, 1);
    PostActions::like(pool, &timmy_post_vote_form).await?;

    // Sara downvotes timmy's post
    let sara_post_vote_form = PostLikeForm::new(inserted_post.id, inserted_sara.id, -1);
    PostActions::like(pool, &sara_post_vote_form).await?;

    let mut expected_post_vote_views = [
      VoteView {
        creator: inserted_sara.clone(),
        creator_banned_from_category: false,
        score: -1,
      },
      VoteView {
        creator: inserted_timmy.clone(),
        creator_banned_from_category: false,
        score: 1,
      },
    ];
    expected_post_vote_views[1].creator.post_count = 1;
    expected_post_vote_views[1].creator.comment_count = 1;

    let read_post_vote_views =
      VoteView::list_for_post(pool, inserted_post.id, None, None, None).await?;
    assert_eq!(read_post_vote_views, expected_post_vote_views);

    // Timothy votes down his own comment
    let timmy_comment_vote_form = CommentLikeForm::new(inserted_timmy.id, inserted_comment.id, -1);
    CommentActions::like(pool, &timmy_comment_vote_form).await?;

    // Sara upvotes timmy's comment
    let sara_comment_vote_form = CommentLikeForm::new(inserted_sara.id, inserted_comment.id, 1);
    CommentActions::like(pool, &sara_comment_vote_form).await?;

    let mut expected_comment_vote_views = [
      VoteView {
        creator: inserted_timmy.clone(),
        creator_banned_from_category: false,
        score: -1,
      },
      VoteView {
        creator: inserted_sara.clone(),
        creator_banned_from_category: false,
        score: 1,
      },
    ];
    expected_comment_vote_views[0].creator.post_count = 1;
    expected_comment_vote_views[0].creator.comment_count = 1;

    let read_comment_vote_views =
      VoteView::list_for_comment(pool, inserted_comment.id, None, None, None).await?;
    assert_eq!(read_comment_vote_views, expected_comment_vote_views);

    // Make sure creator_banned_from_category is true
    let read_comment_vote_views_after_ban =
      VoteView::list_for_comment(pool, inserted_comment.id, None, None, None).await?;

    assert!(read_comment_vote_views_after_ban
      .first()
      .is_some_and(|c| c.creator_banned_from_category));

    let read_post_vote_views_after_ban =
      VoteView::list_for_post(pool, inserted_post.id, None, None, None).await?;

    assert!(read_post_vote_views_after_ban
      .get(1)
      .is_some_and(|p| p.creator_banned_from_category));

    // Cleanup
    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }
}
