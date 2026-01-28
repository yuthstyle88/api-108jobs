use crate::{
  AdminAllowInstanceView,
  AdminBlockInstanceView,
  AdminPurgeCommentView,
  AdminPurgeCategoryView,
  AdminPurgePersonView,
  AdminPurgePostView,
  ModAddCategoryView,
  ModAddView,
  ModBanFromCategoryView,
  ModBanView,
  ModChangeCategoryVisibilityView,
  ModFeaturePostView,
  ModLockPostView,
  ModRemoveCommentView,
  ModRemoveCategoryView,
  ModRemovePostView,
  ModTransferCategoryView,
  ModlogCombinedView,
  ModlogCombinedViewInternal,
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
use app_108jobs_db_schema::{
    aliases,
    impls::local_user::LocalUserOptionHelper,
    newtypes::{CommentId, CategoryId, PaginationCursor, PersonId, PostId},
    source::{
    combined::modlog::{modlog_combined_keys as key, ModlogCombined},
    local_user::LocalUser,
  },
    traits::{InternalToCombinedView, PaginationCursorBuilder},
    utils::{
    get_conn,
    limit_fetch,
    paginate,
    queries::{filter_is_subscribed, filter_not_unlisted_or_is_subscribed},
    DbPool,
  },
    ModlogActionType,
};
use app_108jobs_db_schema_file::{
  enums::ListingType,
  schema::{
      admin_allow_instance,
      admin_block_instance,
      admin_purge_comment,
      admin_purge_category,
      admin_purge_person,
      admin_purge_post,
      comment,
      category,
      category_actions,
      instance,
      mod_add,
      mod_add_category,
      mod_ban,
      mod_ban_from_category,
      mod_change_category_visibility,
      mod_feature_post,
      mod_lock_post,
      mod_remove_comment,
      mod_remove_category,
      mod_remove_post,
      mod_transfer_category,
      modlog_combined,
      person,
      post,
  },
};
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};

impl ModlogCombinedViewInternal {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: Option<PersonId>) -> _ {
    // The modded / other person
    let other_person = aliases::person1.field(person::id);

    // The query for the admin / mod person
    // It needs an OR condition to every mod table
    // After this you can use person::id to refer to the moderator
    let moderator_join = person::table.on(
      admin_allow_instance::admin_person_id
        .eq(person::id)
        .or(admin_block_instance::admin_person_id.eq(person::id))
        .or(admin_purge_comment::admin_person_id.eq(person::id))
        .or(admin_purge_category::admin_person_id.eq(person::id))
        .or(admin_purge_person::admin_person_id.eq(person::id))
        .or(admin_purge_post::admin_person_id.eq(person::id))
        .or(mod_add::mod_person_id.eq(person::id))
        .or(mod_add_category::mod_person_id.eq(person::id))
        .or(mod_ban::mod_person_id.eq(person::id))
        .or(mod_ban_from_category::mod_person_id.eq(person::id))
        .or(mod_feature_post::mod_person_id.eq(person::id))
        .or(mod_change_category_visibility::mod_person_id.eq(person::id))
        .or(mod_lock_post::mod_person_id.eq(person::id))
        .or(mod_remove_comment::mod_person_id.eq(person::id))
        .or(mod_remove_category::mod_person_id.eq(person::id))
        .or(mod_remove_post::mod_person_id.eq(person::id))
        .or(mod_transfer_category::mod_person_id.eq(person::id)),
    );

    let other_person_join = aliases::person1.on(
      mod_add::other_person_id
        .eq(other_person)
        .or(mod_add_category::other_person_id.eq(other_person))
        .or(mod_ban::other_person_id.eq(other_person))
        .or(mod_ban_from_category::other_person_id.eq(other_person))
        // Some tables don't have the other_person_id directly, so you need to join
        .or(
          mod_feature_post::id
            .is_not_null()
            .and(post::creator_id.eq(other_person)),
        )
        .or(
          mod_lock_post::id
            .is_not_null()
            .and(post::creator_id.eq(other_person)),
        )
        .or(
          mod_remove_comment::id
            .is_not_null()
            .and(comment::creator_id.eq(other_person)),
        )
        .or(
          mod_remove_post::id
            .is_not_null()
            .and(post::creator_id.eq(other_person)),
        )
        .or(mod_transfer_category::other_person_id.eq(other_person)),
    );

    let comment_join = comment::table.on(mod_remove_comment::comment_id.eq(comment::id));

    let post_join = post::table.on(
      admin_purge_comment::post_id
        .eq(post::id)
        .or(mod_feature_post::post_id.eq(post::id))
        .or(mod_lock_post::post_id.eq(post::id))
        .or(
          mod_remove_comment::id
            .is_not_null()
            .and(comment::post_id.eq(post::id)),
        )
        .or(mod_remove_post::post_id.eq(post::id)),
    );

    let category_join = category::table.on(
      admin_purge_post::category_id
        .eq(category::id)
        .or(mod_add_category::category_id.eq(category::id))
        .or(mod_ban_from_category::category_id.eq(category::id))
        .or(
          mod_feature_post::id
            .is_not_null()
            .and(post::category_id.is_null().or(category::id.nullable().eq(post::category_id))),
        )
        .or(mod_change_category_visibility::category_id.eq(category::id))
        .or(
          mod_lock_post::id
            .is_not_null()
            .and(post::category_id.is_null().or(category::id.nullable().eq(post::category_id))),
        )
        .or(
          mod_remove_comment::id
            .is_not_null()
            .and(post::category_id.is_null().or(category::id.nullable().eq(post::category_id))),
        )
        .or(mod_remove_category::category_id.eq(category::id))
        .or(
          mod_remove_post::id
            .is_not_null()
            .and(post::category_id.is_null().or(category::id.nullable().eq(post::category_id))),
        )
        .or(mod_transfer_category::category_id.eq(category::id)),
    );

    let instance_join = instance::table.on(
      admin_allow_instance::instance_id
        .eq(instance::id)
        .or(admin_block_instance::instance_id.eq(instance::id)),
    );

    let category_actions_join = category_actions::table.on(
        category_actions::category_id
        .eq(category::id)
        .and(category_actions::person_id.nullable().eq(my_person_id)),
    );

    modlog_combined::table
      .left_join(admin_allow_instance::table)
      .left_join(admin_block_instance::table)
      .left_join(admin_purge_comment::table)
      .left_join(admin_purge_category::table)
      .left_join(admin_purge_person::table)
      .left_join(admin_purge_post::table)
      .left_join(mod_add::table)
      .left_join(mod_add_category::table)
      .left_join(mod_ban::table)
      .left_join(mod_ban_from_category::table)
      .left_join(mod_feature_post::table)
      .left_join(mod_change_category_visibility::table)
      .left_join(mod_lock_post::table)
      .left_join(mod_remove_comment::table)
      .left_join(mod_remove_category::table)
      .left_join(mod_remove_post::table)
      .left_join(mod_transfer_category::table)
      .left_join(moderator_join)
      .left_join(comment_join)
      .left_join(post_join)
      .left_join(category_join)
      .left_join(instance_join)
      .left_join(other_person_join)
      .left_join(category_actions_join)
  }
}

impl PaginationCursorBuilder for ModlogCombinedView {
  type CursorData = ModlogCombined;
  fn to_cursor(&self) -> PaginationCursor {
    use ModlogCombinedView::*;
    let (prefix, id) = match &self {
      AdminAllowInstance(v) => ('A', v.admin_allow_instance.id.0),
      AdminBlockInstance(v) => ('B', v.admin_block_instance.id.0),
      AdminPurgeComment(v) => ('C', v.admin_purge_comment.id.0),
      AdminPurgeCategory(v) => ('D', v.admin_purge_category.id.0),
      AdminPurgePerson(v) => ('E', v.admin_purge_person.id.0),
      AdminPurgePost(v) => ('F', v.admin_purge_post.id.0),
      ModAdd(v) => ('G', v.mod_add.id.0),
      ModAddCategory(v) => ('H', v.mod_add_category.id.0),
      ModBan(v) => ('I', v.mod_ban.id.0),
      ModBanFromCategory(v) => ('J', v.mod_ban_from_category.id.0),
      ModFeaturePost(v) => ('K', v.mod_feature_post.id.0),
      ModChangeCategoryVisibility(v) => ('L', v.mod_change_category_visibility.id.0),
      ModLockPost(v) => ('M', v.mod_lock_post.id.0),
      ModRemoveComment(v) => ('N', v.mod_remove_comment.id.0),
      ModRemoveCategory(v) => ('O', v.mod_remove_category.id.0),
      ModRemovePost(v) => ('P', v.mod_remove_post.id.0),
      ModTransferCategory(v) => ('Q', v.mod_transfer_category.id.0),
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

    let mut query = modlog_combined::table
      .select(Self::CursorData::as_select())
      .into_boxed();

    query = match prefix {
      'A' => query.filter(modlog_combined::admin_allow_instance_id.eq(id)),
      'B' => query.filter(modlog_combined::admin_block_instance_id.eq(id)),
      'C' => query.filter(modlog_combined::admin_purge_comment_id.eq(id)),
      'D' => query.filter(modlog_combined::admin_purge_category_id.eq(id)),
      'E' => query.filter(modlog_combined::admin_purge_person_id.eq(id)),
      'F' => query.filter(modlog_combined::admin_purge_post_id.eq(id)),
      'G' => query.filter(modlog_combined::mod_add_id.eq(id)),
      'H' => query.filter(modlog_combined::mod_add_category_id.eq(id)),
      'I' => query.filter(modlog_combined::mod_ban_id.eq(id)),
      'J' => query.filter(modlog_combined::mod_ban_from_category_id.eq(id)),
      'K' => query.filter(modlog_combined::mod_feature_post_id.eq(id)),
      'L' => query.filter(modlog_combined::mod_change_category_visibility_id.eq(id)),
      'M' => query.filter(modlog_combined::mod_lock_post_id.eq(id)),
      'N' => query.filter(modlog_combined::mod_remove_comment_id.eq(id)),
      'O' => query.filter(modlog_combined::mod_remove_category_id.eq(id)),
      'P' => query.filter(modlog_combined::mod_remove_post_id.eq(id)),
      'Q' => query.filter(modlog_combined::mod_transfer_category_id.eq(id)),
      _ => return Err(FastJobErrorType::CouldntParsePaginationToken.into()),
    };

    let token = query.first(conn).await?;

    Ok(token)
  }
}

#[derive(Default)]
/// Querying / filtering the modlog.
pub struct ModlogCombinedQuery<'a> {
  pub type_: Option<ModlogActionType>,
  pub listing_type: Option<ListingType>,
  pub comment_id: Option<CommentId>,
  pub post_id: Option<PostId>,
  pub category_id: Option<CategoryId>,
  pub hide_modlog_names: Option<bool>,
  pub local_user: Option<&'a LocalUser>,
  pub mod_person_id: Option<PersonId>,
  pub other_person_id: Option<PersonId>,
  pub cursor_data: Option<ModlogCombined>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

impl ModlogCombinedQuery<'_> {
  pub async fn list(self, pool: &mut DbPool<'_>) -> FastJobResult<Vec<ModlogCombinedView>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(self.limit)?;

    let other_person = aliases::person1.field(person::id);
    let my_person_id = self.local_user.person_id();

    let mut query = ModlogCombinedViewInternal::joins(my_person_id)
      .select(ModlogCombinedViewInternal::as_select())
      .limit(limit)
      .into_boxed();

    if let Some(mod_person_id) = self.mod_person_id {
      query = query.filter(person::id.eq(mod_person_id));
    };

    if let Some(other_person_id) = self.other_person_id {
      query = query.filter(other_person.eq(other_person_id));
    };

    if let Some(category_id) = self.category_id {
      query = query.filter(category::id.eq(category_id))
    }

    if let Some(post_id) = self.post_id {
      query = query.filter(post::id.eq(post_id))
    }

    if let Some(comment_id) = self.comment_id {
      query = query.filter(comment::id.eq(comment_id))
    }

    if let Some(type_) = self.type_ {
      use app_108jobs_db_schema::ModlogActionType::*;
      query = match type_ {
        All => query,
        ModRemovePost => query.filter(modlog_combined::mod_remove_post_id.is_not_null()),
        ModLockPost => query.filter(modlog_combined::mod_lock_post_id.is_not_null()),
        ModFeaturePost => query.filter(modlog_combined::mod_feature_post_id.is_not_null()),
        ModRemoveComment => query.filter(modlog_combined::mod_remove_comment_id.is_not_null()),
        ModRemovecategory => query.filter(modlog_combined::mod_remove_category_id.is_not_null()),
        ModBanFromcategory => {
          query.filter(modlog_combined::mod_ban_from_category_id.is_not_null())
        }
        ModAddcategory => query.filter(modlog_combined::mod_add_category_id.is_not_null()),
        ModTransfercategory => {
          query.filter(modlog_combined::mod_transfer_category_id.is_not_null())
        }
        ModAdd => query.filter(modlog_combined::mod_add_id.is_not_null()),
        ModBan => query.filter(modlog_combined::mod_ban_id.is_not_null()),
        ModChangecategoryVisibility => {
          query.filter(modlog_combined::mod_change_category_visibility_id.is_not_null())
        }
        AdminPurgePerson => query.filter(modlog_combined::admin_purge_person_id.is_not_null()),
        AdminPurgecategory => {
          query.filter(modlog_combined::admin_purge_category_id.is_not_null())
        }
        AdminPurgePost => query.filter(modlog_combined::admin_purge_post_id.is_not_null()),
        AdminPurgeComment => query.filter(modlog_combined::admin_purge_comment_id.is_not_null()),
        AdminBlockInstance => query.filter(modlog_combined::admin_block_instance_id.is_not_null()),
        AdminAllowInstance => query.filter(modlog_combined::admin_allow_instance_id.is_not_null()),
      }
    }

    query = match self.listing_type.unwrap_or(ListingType::All) {
      ListingType::All => query,
      ListingType::Subscribed => query.filter(filter_is_subscribed()),
      ListingType::Local => query
        .filter(category::local.eq(true))
        .filter(filter_not_unlisted_or_is_subscribed()),
      ListingType::ModeratorView => {
        query.filter(category_actions::became_moderator_at.is_not_null())
      }
    };

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
      .load::<ModlogCombinedViewInternal>(conn)
      .await?;

    let hide_modlog_names = self.hide_modlog_names.unwrap_or_default();

    // Map the query results to the enum
    let out = res
      .into_iter()
      .map(|u| u.hide_mod_name(hide_modlog_names))
      .filter_map(InternalToCombinedView::map_to_enum)
      .collect();

    Ok(out)
  }
}

impl ModlogCombinedViewInternal {
  /// Hides modlog names by setting the moderator to None.
  fn hide_mod_name(self, hide_modlog_names: bool) -> Self {
    if hide_modlog_names {
      Self {
        moderator: None,
        ..self
      }
    } else {
      self
    }
  }
}

impl InternalToCombinedView for ModlogCombinedViewInternal {
  type CombinedView = ModlogCombinedView;

  fn map_to_enum(self) -> Option<Self::CombinedView> {
    // Use for a short alias
    let v = self;

    if let (Some(admin_allow_instance), Some(instance)) =
      (v.admin_allow_instance, v.instance.clone())
    {
      Some(ModlogCombinedView::AdminAllowInstance(
        AdminAllowInstanceView {
          admin_allow_instance,
          instance,
          admin: v.moderator,
        },
      ))
    } else if let (Some(admin_block_instance), Some(instance)) =
      (v.admin_block_instance, v.instance)
    {
      Some(ModlogCombinedView::AdminBlockInstance(
        AdminBlockInstanceView {
          admin_block_instance,
          instance,
          admin: v.moderator,
        },
      ))
    } else if let (Some(admin_purge_comment), Some(post)) = (v.admin_purge_comment, v.post.clone())
    {
      Some(ModlogCombinedView::AdminPurgeComment(
        AdminPurgeCommentView {
          admin_purge_comment,
          post,
          admin: v.moderator,
        },
      ))
    } else if let Some(admin_purge_category) = v.admin_purge_category {
      Some(ModlogCombinedView::AdminPurgeCategory(
        AdminPurgeCategoryView {
          admin_purge_category,
          admin: v.moderator,
        },
      ))
    } else if let Some(admin_purge_person) = v.admin_purge_person {
      Some(ModlogCombinedView::AdminPurgePerson(AdminPurgePersonView {
        admin_purge_person,
        admin: v.moderator,
      }))
    } else if let Some(admin_purge_post) = v.admin_purge_post {
      Some(ModlogCombinedView::AdminPurgePost(AdminPurgePostView {
        admin_purge_post,
        admin: v.moderator,
        category: v.category.clone(),
      }))
    } else if let (Some(mod_add), Some(other_person)) = (v.mod_add, v.other_person.clone()) {
      Some(ModlogCombinedView::ModAdd(ModAddView {
        mod_add,
        moderator: v.moderator,
        other_person,
      }))
    } else if let (Some(mod_add_category), Some(other_person), Some(category)) = (
      v.mod_add_category,
      v.other_person.clone(),
      v.category.clone(),
    ) {
      Some(ModlogCombinedView::ModAddCategory(ModAddCategoryView {
        mod_add_category,
        moderator: v.moderator,
        other_person,
        category,
      }))
    } else if let (Some(mod_ban), Some(other_person)) = (v.mod_ban, v.other_person.clone()) {
      Some(ModlogCombinedView::ModBan(ModBanView {
        mod_ban,
        moderator: v.moderator,
        other_person,
      }))
    } else if let (Some(mod_ban_from_category), Some(other_person), Some(category)) = (
      v.mod_ban_from_category,
      v.other_person.clone(),
      v.category.clone(),
    ) {
      Some(ModlogCombinedView::ModBanFromCategory(
        ModBanFromCategoryView {
          mod_ban_from_category,
          moderator: v.moderator,
          other_person,
          category,
        },
      ))
    } else if let (Some(mod_feature_post), Some(other_person), Some(post)) = (
      v.mod_feature_post,
      v.other_person.clone(),
      v.post.clone(),
    ) {
      Some(ModlogCombinedView::ModFeaturePost(ModFeaturePostView {
        mod_feature_post,
        moderator: v.moderator,
        other_person,
        category: v.category.clone(),
        post,
      }))
    } else if let (Some(mod_change_category_visibility), Some(category)) =
      (v.mod_change_category_visibility, v.category.clone())
    {
      Some(ModlogCombinedView::ModChangeCategoryVisibility(
        ModChangeCategoryVisibilityView {
          mod_change_category_visibility,
          moderator: v.moderator,
          category,
        },
      ))
    } else if let (Some(mod_lock_post), Some(other_person), Some(post)) = (
      v.mod_lock_post,
      v.other_person.clone(),
      v.post.clone(),
    ) {
      Some(ModlogCombinedView::ModLockPost(ModLockPostView {
        mod_lock_post,
        moderator: v.moderator,
        other_person,
        category: v.category.clone(),
        post,
      }))
    } else if let (
      Some(mod_remove_comment),
      Some(other_person),
      Some(post),
      Some(comment),
    ) = (
      v.mod_remove_comment,
      v.other_person.clone(),
      v.post.clone(),
      v.comment,
    ) {
      Some(ModlogCombinedView::ModRemoveComment(ModRemoveCommentView {
        mod_remove_comment,
        moderator: v.moderator,
        other_person,
        category: v.category.clone(),
        post,
        comment,
      }))
    } else if let (Some(mod_remove_category), Some(category)) =
      (v.mod_remove_category, v.category.clone())
    {
      Some(ModlogCombinedView::ModRemoveCategory(
        ModRemoveCategoryView {
          mod_remove_category,
          moderator: v.moderator,
          category,
        },
      ))
    } else if let (Some(mod_remove_post), Some(other_person), Some(post)) = (
      v.mod_remove_post,
      v.other_person.clone(),
      v.post.clone(),
    ) {
      Some(ModlogCombinedView::ModRemovePost(ModRemovePostView {
        mod_remove_post,
        moderator: v.moderator,
        other_person,
        category: v.category.clone(),
        post,
      }))
    } else if let (Some(mod_transfer_category), Some(other_person), Some(category)) = (
      v.mod_transfer_category,
      v.other_person.clone(),
      v.category.clone(),
    ) {
      Some(ModlogCombinedView::ModTransferCategory(
        ModTransferCategoryView {
          mod_transfer_category,
          moderator: v.moderator,
          other_person,
          category,
        },
      ))
    } else {
      None
    }
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use crate::{impls::ModlogCombinedQuery, ModlogCombinedView};
  use app_108jobs_db_schema::{
    newtypes::PersonId,
    source::{
        comment::{Comment, CommentInsertForm},
        category::{category, CategoryInsertForm},
        instance::Instance,
        mod_log::{
        admin::{
          AdminAllowInstance,
          AdminAllowInstanceForm,
          AdminBlockInstance,
          AdminBlockInstanceForm,
          AdminPurgeComment,
          AdminPurgeCommentForm,
          AdminPurgecategory,
          AdminPurgecategoryForm,
          AdminPurgePerson,
          AdminPurgePersonForm,
          AdminPurgePost,
          AdminPurgePostForm,
        },
        moderator::{
          ModAdd,
          ModAddcategory,
          ModAddcategoryForm,
          ModAddForm,
          ModBan,
          ModBanForm,
          ModBanFromcategory,
          ModBanFromcategoryForm,
          ModChangecategoryVisibility,
          ModChangecategoryVisibilityForm,
          ModFeaturePost,
          ModFeaturePostForm,
          ModLockPost,
          ModLockPostForm,
          ModRemoveComment,
          ModRemoveCommentForm,
          ModRemovecategory,
          ModRemovecategoryForm,
          ModRemovePost,
          ModRemovePostForm,
          ModTransferCategory,
          ModTransfercategoryForm,
        },
      },
        person::{Person, PersonInsertForm},
        post::{Post, PostInsertForm},
    },
    traits::Crud,
    utils::{build_db_pool_for_tests, DbPool},
    ModlogActionType,
  };
  use app_108jobs_db_schema_file::enums::categoryVisibility;
  use app_108jobs_utils::error::FastJobResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;
  use app_108jobs_db_schema::newtypes::DbUrl;

  struct Data {
    instance: Instance,
    timmy: Person,
    sara: Person,
    jessica: Person,
    category: category,
    category_2: category,
    post: Post,
    post_2: Post,
    comment: Comment,
    comment_2: Comment,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> FastJobResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let timmy_form = PersonInsertForm::test_form(instance.id, "timmy_rcv");
    let timmy = Person::create(pool, &timmy_form).await?;

    let sara_form = PersonInsertForm::test_form(instance.id, "sara_rcv");
    let sara = Person::create(pool, &sara_form).await?;

    let jessica_form = PersonInsertForm::test_form(instance.id, "jessica_mrv");
    let jessica = Person::create(pool, &jessica_form).await?;

    let category_form = CategoryInsertForm::new(
      instance.id,
      "test category crv".to_string(),
      "nada".to_owned(),
    );
    let category = category::create(pool, &category_form).await?;

    let category_form_2 = CategoryInsertForm::new(
      instance.id,
      "test category crv 2".to_string(),
      "nada".to_owned(),
    );
    let category_2 = category::create(pool, &category_form_2).await?;

    let post_form = PostInsertForm::new("A test post crv".into(), timmy.id, category.id);
    let post = Post::create(pool, &post_form).await?;

    let new_post_2 = PostInsertForm::new("A test post crv 2".into(), sara.id, category_2.id);
    let post_2 = Post::create(pool, &new_post_2).await?;

    // Timmy creates a comment
    let comment_form = CommentInsertForm::new(timmy.id, post.id, "A test comment rv".into());
    let comment = Comment::create(pool, &comment_form, ).await?;

    // jessica creates a comment
    let comment_form_2 =
      CommentInsertForm::new(jessica.id, post_2.id, "A test comment rv 2".into());
    let comment_2 = Comment::create(pool, &comment_form_2, ).await?;

    Ok(Data {
      instance,
      timmy,
      sara,
      jessica,
      category,
      category_2,
      post,
      post_2,
      comment,
      comment_2,
    })
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> FastJobResult<()> {
    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn admin_types() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let form = AdminAllowInstanceForm {
      instance_id: data.instance.id,
      admin_person_id: data.timmy.id,
      allowed: true,
      reason: None,
    };
    AdminAllowInstance::create(pool, &form).await?;

    let form = AdminBlockInstanceForm {
      instance_id: data.instance.id,
      admin_person_id: data.timmy.id,
      blocked: true,
      reason: None,
    };
    AdminBlockInstance::create(pool, &form).await?;

    let form = AdminPurgeCommentForm {
      admin_person_id: data.timmy.id,
      post_id: data.post.id,
      reason: None,
    };
    AdminPurgeComment::create(pool, &form).await?;

    let form = AdminPurgecategoryForm {
      admin_person_id: data.timmy.id,
      reason: None,
    };
    AdminPurgecategory::create(pool, &form).await?;

    let form = AdminPurgePersonForm {
      admin_person_id: data.timmy.id,
      reason: None,
    };
    AdminPurgePerson::create(pool, &form).await?;

    let form = AdminPurgePostForm {
      admin_person_id: data.timmy.id,
      category_id: data.category.id,
      reason: None,
    };
    AdminPurgePost::create(pool, &form).await?;

    let form = ModChangecategoryVisibilityForm {
      mod_person_id: data.timmy.id,
      category_id: data.category.id,
      visibility: categoryVisibility::Unlisted,
    };
    ModChangecategoryVisibility::create(pool, &form).await?;

    // A 2nd mod hide category, but to a different category, and with jessica
    let form = ModChangecategoryVisibilityForm {
      mod_person_id: data.jessica.id,
      category_id: data.category_2.id,
      visibility: categoryVisibility::Unlisted,
    };
    ModChangecategoryVisibility::create(pool, &form).await?;

    let modlog = ModlogCombinedQuery::default().list(pool).await?;
    assert_eq!(8, modlog.len());

    if let ModlogCombinedView::ModChangecategoryVisibility(v) = &modlog[0] {
      assert_eq!(
        data.category_2.id,
        v.mod_change_category_visibility.category_id
      );
      assert_eq!(data.category_2.id, v.category.id);
      assert_eq!(
        data.jessica.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModChangecategoryVisibility(v) = &modlog[1] {
      assert_eq!(
        data.category.id,
        v.mod_change_category_visibility.category_id
      );
      assert_eq!(data.category.id, v.category.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::AdminPurgePost(v) = &modlog[2] {
      assert_eq!(data.category.id, v.admin_purge_post.category_id);
      assert_eq!(data.category.id, v.category.id);
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::AdminPurgePerson(v) = &modlog[3] {
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::AdminPurgecategory(v) = &modlog[4] {
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::AdminPurgeComment(v) = &modlog[5] {
      assert_eq!(data.post.id, v.admin_purge_comment.post_id);
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    // Make sure the report types are correct
    if let ModlogCombinedView::AdminBlockInstance(v) = &modlog[6] {
      assert_eq!(data.instance.id, v.admin_block_instance.instance_id);
      assert_eq!(data.instance.id, v.instance.id);
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::AdminAllowInstance(v) = &modlog[7] {
      assert_eq!(data.instance.id, v.admin_allow_instance.instance_id);
      assert_eq!(data.instance.id, v.instance.id);
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    // Filter by admin
    let modlog_admin_filter = ModlogCombinedQuery {
      mod_person_id: Some(data.timmy.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    // Only one is jessica
    assert_eq!(7, modlog_admin_filter.len());

    // Filter by category
    let modlog_category_filter = ModlogCombinedQuery {
      category_id: Some(data.category.id),
      ..Default::default()
    }
    .list(pool)
    .await?;

    // Should be 2, and not jessicas
    assert_eq!(2, modlog_category_filter.len());

    // Filter by type
    let modlog_type_filter = ModlogCombinedQuery {
      type_: Some(ModlogActionType::ModChangecategoryVisibility),
      ..Default::default()
    }
    .list(pool)
    .await?;

    // 2 of these, one is jessicas
    assert_eq!(2, modlog_type_filter.len());

    if let ModlogCombinedView::ModChangecategoryVisibility(v) = &modlog_type_filter[0] {
      assert_eq!(
        data.category_2.id,
        v.mod_change_category_visibility.category_id
      );
      assert_eq!(data.category_2.id, v.category.id);
      assert_eq!(
        data.jessica.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModChangecategoryVisibility(v) = &modlog_type_filter[1] {
      assert_eq!(
        data.category.id,
        v.mod_change_category_visibility.category_id
      );
      assert_eq!(data.category.id, v.category.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn mod_types() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let form = ModAddForm {
      mod_person_id: data.timmy.id,
      other_person_id: data.jessica.id,
      removed: Some(false),
    };
    ModAdd::create(pool, &form).await?;

    let form = ModAddcategoryForm {
      mod_person_id: data.timmy.id,
      other_person_id: data.jessica.id,
      category_id: data.category.id,
      removed: Some(false),
    };
    ModAddcategory::create(pool, &form).await?;

    let form = ModBanForm {
      mod_person_id: data.timmy.id,
      other_person_id: data.jessica.id,
      banned: Some(true),
      reason: None,
      expires_at: None,
      instance_id: data.instance.id,
    };
    ModBan::create(pool, &form).await?;

    let form = ModBanFromcategoryForm {
      mod_person_id: data.timmy.id,
      other_person_id: data.jessica.id,
      category_id: data.category.id,
      banned: Some(true),
      reason: None,
      expires_at: None,
    };
    ModBanFromcategory::create(pool, &form).await?;

    let form = ModFeaturePostForm {
      mod_person_id: data.timmy.id,
      post_id: data.post.id,
      featured: Some(true),
      is_featured_category: None,
    };
    ModFeaturePost::create(pool, &form).await?;

    let form = ModLockPostForm {
      mod_person_id: data.timmy.id,
      post_id: data.post.id,
      locked: Some(true),
      reason: None,
    };
    ModLockPost::create(pool, &form).await?;

    let form = ModRemoveCommentForm {
      mod_person_id: data.timmy.id,
      comment_id: data.comment.id,
      removed: Some(true),
      reason: None,
    };
    ModRemoveComment::create(pool, &form).await?;

    let form = ModRemovecategoryForm {
      mod_person_id: data.timmy.id,
      category_id: data.category.id,
      removed: Some(true),
      reason: None,
    };
    ModRemovecategory::create(pool, &form).await?;

    let form = ModRemovePostForm {
      mod_person_id: data.timmy.id,
      post_id: data.post.id,
      removed: Some(true),
      reason: None,
    };
    ModRemovePost::create(pool, &form).await?;

    let form = ModTransfercategoryForm {
      mod_person_id: data.timmy.id,
      other_person_id: data.jessica.id,
      category_id: data.category.id,
    };
    ModTransfercategory::create(pool, &form).await?;

    // A few extra ones to test different filters
    let form = ModTransfercategoryForm {
      mod_person_id: data.jessica.id,
      other_person_id: data.sara.id,
      category_id: data.category_2.id,
    };
    ModTransfercategory::create(pool, &form).await?;

    let form = ModRemovePostForm {
      mod_person_id: data.jessica.id,
      post_id: data.post_2.id,
      removed: Some(true),
      reason: None,
    };
    ModRemovePost::create(pool, &form).await?;

    let form = ModRemoveCommentForm {
      mod_person_id: data.jessica.id,
      comment_id: data.comment_2.id,
      removed: Some(true),
      reason: None,
    };
    ModRemoveComment::create(pool, &form).await?;

    // The all view
    let modlog = ModlogCombinedQuery::default().list(pool).await?;
    assert_eq!(13, modlog.len());

    if let ModlogCombinedView::ModRemoveComment(v) = &modlog[0] {
      assert_eq!(data.comment_2.id, v.mod_remove_comment.comment_id);
      assert_eq!(data.comment_2.id, v.comment.id);
      assert_eq!(data.post_2.id, v.post.id);
      assert_eq!(data.category_2.id, v.category.id);
      assert_eq!(
        data.jessica.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModRemovePost(v) = &modlog[1] {
      assert_eq!(data.post_2.id, v.mod_remove_post.post_id);
      assert_eq!(data.post_2.id, v.post.id);
      assert_eq!(data.sara.id, v.post.creator_id);
      assert_eq!(data.category_2.id, v.category.id);
      assert_eq!(
        data.jessica.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.sara.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModTransfercategory(v) = &modlog[2] {
      assert_eq!(data.category_2.id, v.mod_transfer_category.category_id);
      assert_eq!(data.category_2.id, v.category.id);
      assert_eq!(
        data.jessica.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.sara.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModTransfercategory(v) = &modlog[3] {
      assert_eq!(data.category.id, v.mod_transfer_category.category_id);
      assert_eq!(data.category.id, v.category.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModRemovePost(v) = &modlog[4] {
      assert_eq!(data.post.id, v.mod_remove_post.post_id);
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(data.timmy.id, v.post.creator_id);
      assert_eq!(data.category.id, v.category.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.timmy.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModRemovecategory(v) = &modlog[5] {
      assert_eq!(data.category.id, v.mod_remove_category.category_id);
      assert_eq!(data.category.id, v.category.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModRemoveComment(v) = &modlog[6] {
      assert_eq!(data.comment.id, v.mod_remove_comment.comment_id);
      assert_eq!(data.comment.id, v.comment.id);
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(data.category.id, v.category.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.timmy.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModLockPost(v) = &modlog[7] {
      assert_eq!(data.post.id, v.mod_lock_post.post_id);
      assert!(v.mod_lock_post.locked);
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(data.timmy.id, v.post.creator_id);
      assert_eq!(data.category.id, v.category.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.timmy.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModFeaturePost(v) = &modlog[8] {
      assert_eq!(data.post.id, v.mod_feature_post.post_id);
      assert!(v.mod_feature_post.featured);
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(data.timmy.id, v.post.creator_id);
      assert_eq!(data.category.id, v.category.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.timmy.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModBanFromcategory(v) = &modlog[9] {
      assert_eq!(data.category.id, v.mod_ban_from_category.category_id);
      assert_eq!(data.category.id, v.category.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModBan(v) = &modlog[10] {
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModAddcategory(v) = &modlog[11] {
      assert_eq!(data.category.id, v.mod_add_category.category_id);
      assert_eq!(data.category.id, v.category.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModAdd(v) = &modlog[12] {
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    // Filter by moderator
    let modlog_mod_timmy_filter = ModlogCombinedQuery {
      mod_person_id: Some(data.timmy.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(10, modlog_mod_timmy_filter.len());

    let modlog_mod_jessica_filter = ModlogCombinedQuery {
      mod_person_id: Some(data.jessica.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(3, modlog_mod_jessica_filter.len());

    // Filter by other_person
    // Gets a little complicated because things aren't directly linked,
    // you have to go into the item to see who created it.

    let modlog_modded_timmy_filter = ModlogCombinedQuery {
      other_person_id: Some(data.timmy.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(4, modlog_modded_timmy_filter.len());

    let modlog_modded_jessica_filter = ModlogCombinedQuery {
      other_person_id: Some(data.jessica.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(6, modlog_modded_jessica_filter.len());

    let modlog_modded_sara_filter = ModlogCombinedQuery {
      other_person_id: Some(data.sara.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(2, modlog_modded_sara_filter.len());

    // Filter by category
    let modlog_category_filter = ModlogCombinedQuery {
      category_id: Some(data.category.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(8, modlog_category_filter.len());

    let modlog_category_2_filter = ModlogCombinedQuery {
      category_id: Some(data.category_2.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(3, modlog_category_2_filter.len());

    // Filter by post
    let modlog_post_filter = ModlogCombinedQuery {
      post_id: Some(data.post.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(4, modlog_post_filter.len());

    let modlog_post_2_filter = ModlogCombinedQuery {
      post_id: Some(data.post_2.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(2, modlog_post_2_filter.len());

    // Filter by comment
    let modlog_comment_filter = ModlogCombinedQuery {
      comment_id: Some(data.comment.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(1, modlog_comment_filter.len());

    let modlog_comment_2_filter = ModlogCombinedQuery {
      comment_id: Some(data.comment_2.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(1, modlog_comment_2_filter.len());

    // Filter by type
    let modlog_type_filter = ModlogCombinedQuery {
      type_: Some(ModlogActionType::ModRemoveComment),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(2, modlog_type_filter.len());

    // Assert that the types are correct
    assert!(matches!(
      modlog_type_filter[0],
      ModlogCombinedView::ModRemoveComment(_)
    ));
    assert!(matches!(
      modlog_type_filter[1],
      ModlogCombinedView::ModRemoveComment(_)
    ));

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn hide_modlog_names() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let form = AdminAllowInstanceForm {
      instance_id: data.instance.id,
      admin_person_id: data.timmy.id,
      allowed: true,
      reason: None,
    };
    AdminAllowInstance::create(pool, &form).await?;

    let modlog = ModlogCombinedQuery::default().list(pool).await?;
    assert_eq!(1, modlog.len());

    if let ModlogCombinedView::AdminAllowInstance(v) = &modlog[0] {
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    // Filter out the names
    let modlog_hide_names_filter = ModlogCombinedQuery {
      hide_modlog_names: Some(true),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(1, modlog_hide_names_filter.len());

    if let ModlogCombinedView::AdminAllowInstance(v) = &modlog_hide_names_filter[0] {
      assert!(v.admin.is_none())
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }
}
