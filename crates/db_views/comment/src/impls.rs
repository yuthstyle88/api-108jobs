use crate::api::{CreateComment, CreateCommentRequest};
use crate::{CommentSlimView, CommentView};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use diesel_ltree::Ltree;
use i_love_jesus::asc_if;
use lemmy_db_schema::impls::local_user::LocalUserOptionHelper;
use lemmy_db_schema::newtypes::CommunityId;
use lemmy_db_schema::source::local_user::LocalUser;
use lemmy_db_schema::{
  newtypes::{CommentId, InstanceId, PaginationCursor, PersonId, PostId},
  source::{
    comment::{comment_keys as key, Comment},
    site::Site,
  },
  traits::{Crud, PaginationCursorBuilder},
  utils::{
    get_conn, limit_fetch, now, paginate,
    queries::{
      creator_community_actions_join, creator_community_instance_actions_join,
      creator_home_instance_actions_join, creator_local_instance_actions_join,
      my_comment_actions_join, my_community_actions_join, my_instance_actions_community_join,
      my_local_user_admin_join, my_person_actions_join,
    },
    seconds_to_pg_interval, DbPool,
  },
};
use lemmy_db_schema_file::{
  enums::{
    CommentSortType::{self, *},
    CommunityFollowerState, CommunityVisibility, ListingType,
  },
  schema::{comment, community, community_actions, person, post},
};
use lemmy_utils::error::{FastJobError, FastJobErrorExt, FastJobErrorType, FastJobResult};

impl PaginationCursorBuilder for CommentView {
  type CursorData = Comment;
  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new_single('C', self.comment.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    let id = cursor.first_id()?;
    Comment::read(pool, CommentId(id)).await
  }
}

impl CommentView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: Option<PersonId>, local_instance_id: InstanceId) -> _ {
    let community_join = community::table.on(post::community_id.eq(community::id));

    let my_community_actions_join: my_community_actions_join =
      my_community_actions_join(my_person_id);
    let my_comment_actions_join: my_comment_actions_join = my_comment_actions_join(my_person_id);
    let my_local_user_admin_join: my_local_user_admin_join = my_local_user_admin_join(my_person_id);
    let my_instance_actions_community_join: my_instance_actions_community_join =
      my_instance_actions_community_join(my_person_id);
    let my_person_actions_join: my_person_actions_join = my_person_actions_join(my_person_id);
    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(local_instance_id);

    comment::table
      .inner_join(person::table)
      .inner_join(post::table)
      .inner_join(community_join)
      .left_join(my_community_actions_join)
      .left_join(my_comment_actions_join)
      .left_join(my_person_actions_join)
      .left_join(my_local_user_admin_join)
      .left_join(my_instance_actions_community_join)
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_community_instance_actions_join())
      .left_join(creator_local_instance_actions_join)
      .left_join(creator_community_actions_join())
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
    my_local_user: Option<&'_ LocalUser>,
    local_instance_id: InstanceId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    let mut query = Self::joins(my_local_user.person_id(), local_instance_id)
      .filter(comment::id.eq(comment_id))
      .select(Self::as_select())
      .into_boxed();

    query = my_local_user.visible_communities_only(query);

    // Check permissions to view private community content.
    // Specifically, if the community is private then only accepted followers may view its
    // content, otherwise it is filtered out. Admins can view private community content
    // without restriction.
    if !my_local_user.is_admin() {
      query = query.filter(
        community::visibility
          .ne(CommunityVisibility::Private)
          .or(community_actions::follow_state.eq(CommunityFollowerState::Accepted)),
      );
    }

    query
      .first::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub fn map_to_slim(self) -> CommentSlimView {
    CommentSlimView {
      comment: self.comment,
      creator: self.creator,
      comment_actions: self.comment_actions,
      person_actions: self.person_actions,
      instance_actions: self.instance_actions,
      creator_is_admin: self.creator_is_admin,
      can_mod: self.can_mod,
      creator_banned: self.creator_banned,
      creator_banned_from_community: self.creator_banned_from_community,
      creator_is_moderator: self.creator_is_moderator,
    }
  }
}
impl TryFrom<CreateCommentRequest> for CreateComment {
  type Error = FastJobError;

  fn try_from(value: CreateCommentRequest) -> Result<Self, Self::Error> {
    Ok(Self {
      content: value.content,
      post_id: value.post_id,
      parent_id: value.parent_id,
      language_id: Some(value.language_id),
    })
  }
}
#[derive(Default)]
pub struct CommentQuery<'a> {
  pub listing_type: Option<ListingType>,
  pub sort: Option<CommentSortType>,
  pub time_range_seconds: Option<i32>,
  pub community_id: Option<CommunityId>,
  pub post_id: Option<PostId>,
  pub parent_path: Option<Ltree>,
  pub local_user: Option<&'a LocalUser>,
  pub max_depth: Option<i32>,
  pub cursor_data: Option<Comment>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

impl CommentQuery<'_> {
  pub async fn list(self, site: &Site, pool: &mut DbPool<'_>) -> FastJobResult<Vec<CommentView>> {
    let conn = &mut get_conn(pool).await?;
    let o = self;

    // Public query - no user-based filtering, only basic joins
    let mut query = CommentView::joins(None, site.instance_id)
      .select(CommentView::as_select())
      .into_boxed();

    // Filter out deleted and removed comments
    query = query
      .filter(comment::deleted.eq(false))
      .filter(comment::removed.eq(false));

    // Only filter by post_id if specified - no user-based filtering
    if let Some(post_id) = o.post_id {
      query = query.filter(comment::post_id.eq(post_id));
    };

    // Community visibility filtering removed - show all comments regardless of community visibility

    // Filter by the time range
    if let Some(time_range_seconds) = o.time_range_seconds {
      query =
        query.filter(comment::published_at.gt(now() - seconds_to_pg_interval(time_range_seconds)));
    }

    // Comments are now flat, no tree structure
    let limit = limit_fetch(o.limit)?;
    query = query.limit(limit);

    // Only sort by ascending for Old
    let sort = o.sort.unwrap_or(Hot);
    let sort_direction = asc_if(sort == Old);

    let mut pq = paginate(query, sort_direction, o.cursor_data, None, o.page_back);

    // Distinguished comments should go first when viewing post
    // Don't do for new / old sorts
    if sort != New && sort != Old && o.post_id.is_some() {
      pq = pq.then_order_by(key::distinguished);
    }

    pq = match sort {
      Hot => pq.then_order_by(key::hot_rank).then_order_by(key::score),
      Controversial => pq.then_order_by(key::controversy_rank),
      Old | New => pq.then_order_by(key::published_at),
      Top => pq.then_order_by(key::score),
    };

    let res = pq.load::<CommentView>(conn).await?;

    Ok(res)
  }
}
