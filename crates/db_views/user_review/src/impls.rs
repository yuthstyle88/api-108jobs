use crate::UserReviewView;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases,
  newtypes::{PaginationCursor, PersonId, UserReviewId, WorkflowId},
  source::user_review::UserReview,
  traits::{Crud, PaginationCursorBuilder},
  utils::{get_conn, limit_fetch, DbPool},
};
use lemmy_db_schema_file::schema::{person, user_review, workflow};
use lemmy_utils::error::{FastJobErrorType, FastJobResult};

impl PaginationCursorBuilder for UserReviewView {
  type CursorData = UserReview;

  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new_single('R', self.review.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    let id = cursor.first_id()?;
    UserReview::read(pool, UserReviewId(id)).await
  }
}

macro_rules! base_user_review_query {
  ($filter:expr) => {
    UserReviewView::joins()
      .filter($filter)
      .select((
        user_review::all_columns,
        aliases::person1.fields(person::all_columns),
        aliases::person2.fields(person::all_columns),
        workflow::all_columns,
      ))
      .into_boxed()
  };
}

#[macro_export]
macro_rules! apply_cursor_pagination {
    ($query:expr, $cursor_data:expr, $page_back:expr) => {
        if let Some(cursor) = $cursor_data {
            if $page_back.unwrap_or(false) {
                $query = $query.filter(user_review::id.lt(cursor.id));
            } else {
                $query = $query.filter(user_review::id.gt(cursor.id));
            }
        }
    };
}


impl UserReviewView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    let reviewer_join =
      aliases::person1.on(user_review::reviewer_id.eq(aliases::person1.field(person::id)));

    let reviewee_join =
      aliases::person2.on(user_review::reviewee_id.eq(aliases::person2.field(person::id)));

    let workflow_join = workflow::table.on(user_review::workflow_id.eq(workflow::id));

    user_review::table
      .inner_join(reviewer_join)
      .inner_join(reviewee_join)
      .inner_join(workflow_join)
  }

  pub async fn read(pool: &mut DbPool<'_>, id: UserReviewId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(user_review::id.eq(id))
      .select((
        user_review::all_columns,
        aliases::person1.fields(person::all_columns),
        aliases::person2.fields(person::all_columns),
        workflow::all_columns,
      ))
      .first(conn)
      .await
      .map_err(|_| FastJobErrorType::NotFound.into())
  }

  pub async fn list_for_workflow(
    pool: &mut DbPool<'_>,
    workflow_id: WorkflowId,
    limit: Option<i64>,
    cursor_data: Option<UserReview>,
    page_back: Option<bool>,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(limit)?;

    let mut query = base_user_review_query!(user_review::workflow_id.eq(workflow_id));

    apply_cursor_pagination!(query, cursor_data, page_back);

    let res = query
      .order_by(user_review::id.desc())
      .limit(limit)
      .load::<Self>(conn)
      .await?;
    Ok(res)
  }

  pub async fn list_for_user(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    limit: Option<i64>,
    cursor_data: Option<UserReview>,
    page_back: Option<bool>,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(limit)?;

    let mut query = base_user_review_query!(user_review::reviewee_id.eq(person_id));

    apply_cursor_pagination!(query, cursor_data, page_back);

    let res = query
      .order_by(user_review::id.desc())
      .limit(limit)
      .load::<Self>(conn)
      .await?;
    Ok(res)
  }
}
