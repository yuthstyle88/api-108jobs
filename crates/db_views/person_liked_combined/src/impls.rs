use crate::{
  CommentView,
  LocalUserView,
  PersonLikedCombinedView,
  PersonLikedCombinedViewInternal,
  PostView,
};
use app_108jobs_core::error::{FastJobErrorType, FastJobResult};
use app_108jobs_db_schema::{
  newtypes::{InstanceId, PaginationCursor, PersonId},
  source::combined::person_liked::{person_liked_combined_keys as key, PersonLikedCombined},
  traits::{InternalToCombinedView, PaginationCursorBuilder},
  utils::{
    get_conn,
    limit_fetch,
    paginate,
    queries::{
      category_join,
      creator_category_actions_join,
      creator_category_instance_actions_join,
      creator_home_instance_actions_join,
      creator_local_instance_actions_join,
      creator_local_user_admin_join,
      image_details_join,
      my_category_actions_join,
      my_comment_actions_join,
      my_instance_actions_person_join,
      my_local_user_admin_join,
      my_person_actions_join,
      my_post_actions_join,
    },
    DbPool,
  },
  LikeType,
  PersonContentType,
};
use app_108jobs_db_schema_file::schema::{
  comment,
  delivery_details,
  person,
  person_liked_combined,
  post,
};
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;

#[derive(Default)]
pub struct PersonLikedCombinedQuery {
  pub type_: Option<PersonContentType>,
  pub like_type: Option<LikeType>,
  pub cursor_data: Option<PersonLikedCombined>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
  pub no_limit: Option<bool>,
}

impl PaginationCursorBuilder for PersonLikedCombinedView {
  type CursorData = PersonLikedCombined;

  fn to_cursor(&self) -> PaginationCursor {
    let (prefix, id) = match &self {
      PersonLikedCombinedView::Comment(v) => ('C', v.comment.id.0),
      PersonLikedCombinedView::Post(v) => ('P', v.post.id.0),
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

    let mut query = person_liked_combined::table
      .select(Self::CursorData::as_select())
      .into_boxed();

    query = match prefix {
      'C' => query.filter(person_liked_combined::comment_id.eq(id)),
      'P' => query.filter(person_liked_combined::post_id.eq(id)),
      _ => return Err(FastJobErrorType::CouldntParsePaginationToken.into()),
    };
    let token = query.first(conn).await?;

    Ok(token)
  }
}

impl PersonLikedCombinedViewInternal {
  #[diesel::dsl::auto_type(no_type_alias)]
  pub(crate) fn joins(my_person_id: PersonId, local_instance_id: InstanceId) -> _ {
    let item_creator = person::id;

    let comment_join =
      comment::table.on(person_liked_combined::comment_id.eq(comment::id.nullable()));

    let post_join = post::table.on(
      person_liked_combined::post_id
        .eq(post::id.nullable())
        .or(comment::post_id.eq(post::id)),
    );

    let item_creator_join = person::table.on(
      comment::creator_id
        .eq(item_creator)
        // Need to filter out the post rows where the post_id given is null
        // Otherwise you'll get duped post rows
        .or(
          post::creator_id
            .eq(item_creator)
            .and(person_liked_combined::post_id.is_not_null()),
        ),
    );

    let my_category_actions_join: my_category_actions_join =
      my_category_actions_join(Some(my_person_id));
    let my_post_actions_join: my_post_actions_join = my_post_actions_join(Some(my_person_id));
    let my_comment_actions_join: my_comment_actions_join =
      my_comment_actions_join(Some(my_person_id));
    let my_local_user_admin_join: my_local_user_admin_join =
      my_local_user_admin_join(Some(my_person_id));
    let my_instance_actions_person_join: my_instance_actions_person_join =
      my_instance_actions_person_join(Some(my_person_id));
    let my_person_actions_join: my_person_actions_join = my_person_actions_join(Some(my_person_id));
    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(local_instance_id);

    person_liked_combined::table
      .left_join(comment_join)
      .inner_join(post_join)
      .inner_join(item_creator_join)
      .inner_join(category_join())
      .left_join(creator_category_actions_join())
      .left_join(my_local_user_admin_join)
      .left_join(creator_local_user_admin_join())
      .left_join(my_category_actions_join)
      .left_join(my_instance_actions_person_join)
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_category_instance_actions_join())
      .left_join(creator_local_instance_actions_join)
      .left_join(my_post_actions_join)
      .left_join(my_person_actions_join)
      .left_join(my_comment_actions_join)
      .left_join(image_details_join())
      .left_join(delivery_details::table.on(delivery_details::post_id.eq(post::id)))
  }
}

impl PersonLikedCombinedQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    user: &LocalUserView,
  ) -> FastJobResult<Vec<PersonLikedCombinedView>> {
    let my_person_id = user.local_user.person_id;
    let local_instance_id = user.person.instance_id;

    let conn = &mut get_conn(pool).await?;

    let mut query = PersonLikedCombinedViewInternal::joins(my_person_id, local_instance_id)
      .filter(person_liked_combined::person_id.eq(my_person_id))
      .select(PersonLikedCombinedViewInternal::as_select())
      .into_boxed();

    if !self.no_limit.unwrap_or_default() {
      let limit = limit_fetch(self.limit)?;
      query = query.limit(limit);
    }

    if let Some(type_) = self.type_ {
      query = match type_ {
        PersonContentType::All => query,
        PersonContentType::Comments => {
          query.filter(person_liked_combined::comment_id.is_not_null())
        }
        PersonContentType::Posts => query.filter(person_liked_combined::post_id.is_not_null()),
      }
    }

    if let Some(like_type) = self.like_type {
      query = match like_type {
        LikeType::All => query,
        LikeType::LikedOnly => query.filter(person_liked_combined::like_score.eq(1)),
        LikeType::DislikedOnly => query.filter(person_liked_combined::like_score.eq(-1)),
      }
    }

    // Sorting by liked desc
    let paginated_query = paginate(
      query,
      SortDirection::Desc,
      self.cursor_data,
      None,
      self.page_back,
    )
    .then_order_by(key::liked_at)
    // Tie breaker
    .then_order_by(key::id);

    let res = paginated_query
      .load::<PersonLikedCombinedViewInternal>(conn)
      .await?;

    // Map the query results to the enum
    let out = res
      .into_iter()
      .filter_map(InternalToCombinedView::map_to_enum)
      .collect();

    Ok(out)
  }
}

impl InternalToCombinedView for PersonLikedCombinedViewInternal {
  type CombinedView = PersonLikedCombinedView;

  fn map_to_enum(self) -> Option<Self::CombinedView> {
    // Use for a short alias
    let v = self;

    if let Some(comment) = v.comment {
      Some(PersonLikedCombinedView::Comment(CommentView {
        comment,
        post: v.post,
        category: Some(v.category),
        creator: v.item_creator,
        category_actions: v.category_actions,
        comment_actions: v.comment_actions,
        person_actions: v.person_actions,
        instance_actions: v.instance_actions,
        creator_is_admin: v.item_creator_is_admin,
        post_tags: v.post_tags,
        can_mod: v.can_mod,
        creator_banned: v.creator_banned,
        creator_is_moderator: v.creator_is_moderator,
        creator_banned_from_category: v.creator_banned_from_category,
      }))
    } else {
      Some(PersonLikedCombinedView::Post(PostView {
        post: v.post,
        category: Some(v.category),
        creator: v.item_creator,
        image_details: v.image_details,
        category_actions: v.category_actions,
        post_actions: v.post_actions,
        person_actions: v.person_actions,
        instance_actions: v.instance_actions,
        creator_is_admin: v.item_creator_is_admin,
        tags: v.post_tags,
        can_mod: v.can_mod,
        creator_banned: v.creator_banned,
        creator_is_moderator: v.creator_is_moderator,
        creator_banned_from_category: v.creator_banned_from_category,
      }))
    }
  }
}
