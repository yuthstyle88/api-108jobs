use crate::VoteView;
use app_108jobs_db_schema::{
  aliases::creator_category_actions,
  newtypes::{CommentId, PaginationCursor, PersonId, PostId},
  source::{comment::CommentActions, post::PostActions},
  utils::{get_conn, limit_fetch, paginate, DbPool},
};
use app_108jobs_db_schema_file::schema::{
  category_actions, comment, comment_actions, person, post, post_actions,
};
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use diesel::{
  BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;

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
