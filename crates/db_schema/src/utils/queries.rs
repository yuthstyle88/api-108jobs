use crate::{
  aliases::{
    creator_category_actions, creator_category_instance_actions, creator_home_instance_actions,
    creator_local_instance_actions, creator_local_user, person1, person2,
  },
  newtypes::{InstanceId, PersonId},
  Person1AliasAllColumnsTuple, Person2AliasAllColumnsTuple,
};
use diesel::{
  dsl::{case_when, exists, not},
  expression::SqlLiteral,
  helper_types::{Eq, NotEq},
  sql_types::Json,
  BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
  PgExpressionMethods, QueryDsl,
};
use app_108jobs_db_schema_file::{
  enums::{CategoryFollowerState, CategoryVisibility},
  schema::{
    category, category_actions, comment, comment_actions, image_details, instance_actions,
    local_user, person, person_actions, post, post_actions, post_tag, tag,
  },
};

/// Hide all content from blocked communities and persons. Content from blocked instances is also
/// hidden, unless the user followed the category explicitly.
#[diesel::dsl::auto_type]
pub fn filter_blocked() -> _ {
  instance_actions::blocked_at
    .is_null()
    .or(category_actions::followed_at.is_not_null())
    .and(category_actions::blocked_at.is_null())
    .and(person_actions::blocked_at.is_null())
}

/// Checks that the creator_local_user is an admin.
#[diesel::dsl::auto_type]
pub fn creator_is_admin() -> _ {
  creator_local_user
    .field(local_user::admin)
    .nullable()
    .is_not_distinct_from(true)
}

/// Checks that the local_user is an admin.
#[diesel::dsl::auto_type]
pub fn local_user_is_admin() -> _ {
  local_user::admin.nullable().is_not_distinct_from(true)
}

/// Checks to see if the comment creator is an admin.
#[diesel::dsl::auto_type]
pub fn comment_creator_is_admin() -> _ {
  exists(
    creator_local_user.filter(
      comment::creator_id
        .eq(creator_local_user.field(local_user::person_id))
        .and(creator_local_user.field(local_user::admin).eq(true)),
    ),
  )
}

#[diesel::dsl::auto_type]
pub fn post_creator_is_admin() -> _ {
  exists(
    creator_local_user.filter(
      post::creator_id
        .eq(creator_local_user.field(local_user::person_id))
        .and(creator_local_user.field(local_user::admin).eq(true)),
    ),
  )
}

#[diesel::dsl::auto_type]
pub fn creator_is_moderator() -> _ {
  creator_category_actions
    .field(category_actions::became_moderator_at)
    .nullable()
    .is_not_null()
}

#[diesel::dsl::auto_type]
pub fn creator_banned_from_category() -> _ {
  creator_category_actions
    .field(category_actions::received_ban_at)
    .nullable()
    .is_not_null()
}

#[diesel::dsl::auto_type]
pub fn creator_home_banned() -> _ {
  creator_home_instance_actions
    .field(instance_actions::received_ban_at)
    .nullable()
    .is_not_null()
}

#[diesel::dsl::auto_type]
/// Checks to see if a user is site banned from any of these places:
/// - Their own instance
/// - The local instance
pub fn creator_banned() -> _ {
  let local_ban = creator_local_instance_actions
    .field(instance_actions::received_ban_at)
    .nullable()
    .is_not_null();
  local_ban.or(creator_home_banned())
}

/// Similar to creator_banned(), but also checks if creator was banned from instance where the
/// category is hosted.
#[diesel::dsl::auto_type]
pub fn creator_banned_within_category() -> _ {
  let category_ban = creator_category_instance_actions
    .field(instance_actions::received_ban_at)
    .nullable()
    .is_not_null();
  creator_banned().or(category_ban)
}

#[diesel::dsl::auto_type]
pub fn creator_local_user_admin_join() -> _ {
  creator_local_user.on(
    person::id
      .eq(creator_local_user.field(local_user::person_id))
      .and(creator_local_user.field(local_user::admin).eq(true)),
  )
}

#[diesel::dsl::auto_type]
fn am_higher_mod() -> _ {
  let i_became_moderator = category_actions::became_moderator_at.nullable();

  let creator_became_moderator = creator_category_actions
    .field(category_actions::became_moderator_at)
    .nullable();

  i_became_moderator.is_not_null().and(
    creator_became_moderator
      .ge(i_became_moderator)
      .is_distinct_from(false),
  )
}

/// Checks to see if you can mod an item.
///
/// Caveat: Since admin status isn't federated or ordered, it can't know whether
/// item creator is a federated admin, or a higher admin.
/// The back-end will reject an action for admin that is higher via
/// LocalUser::is_higher_mod_or_admin_check
#[diesel::dsl::auto_type]
pub fn local_user_can_mod() -> _ {
  local_user_is_admin().or(not(creator_is_admin()).and(am_higher_mod()))
}

/// Checks to see if you can mod a post.
#[diesel::dsl::auto_type]
pub fn local_user_can_mod_post() -> _ {
  local_user_is_admin().or(not(post_creator_is_admin()).and(am_higher_mod()))
}

/// Checks to see if you can mod a comment.
#[diesel::dsl::auto_type]
pub fn local_user_can_mod_comment() -> _ {
  local_user_is_admin().or(not(comment_creator_is_admin()).and(am_higher_mod()))
}

/// A special type of can_mod for communities, which dont have creators.
#[diesel::dsl::auto_type]
pub fn local_user_category_can_mod() -> _ {
  let am_admin = local_user::admin.nullable();
  let am_moderator = category_actions::became_moderator_at
    .nullable()
    .is_not_null();
  am_admin.or(am_moderator).is_not_distinct_from(true)
}

/// Selects the comment columns, but gives an empty string for content when
/// deleted or removed, and you're not a mod/admin.
#[diesel::dsl::auto_type]
pub fn comment_select_remove_deletes() -> _ {
  let deleted_or_removed = comment::deleted.or(comment::removed);

  // You can only view the content if it hasn't been removed, or you can mod.
  let can_view_content = not(deleted_or_removed).or(local_user_can_mod_comment());
  let content = case_when(can_view_content, comment::content).otherwise("");

  (
    comment::id,
    comment::creator_id,
    comment::post_id,
    content,
    comment::removed,
    comment::published_at,
    comment::updated_at,
    comment::deleted,
    comment::ap_id,
    comment::local,
    comment::path,
    comment::distinguished,
    comment::language_id,
    comment::score,
    comment::upvotes,
    comment::downvotes,
    comment::child_count,
    comment::hot_rank,
    comment::controversy_rank,
    comment::report_count,
    comment::unresolved_report_count,
    comment::pending,
  )
}

#[diesel::dsl::auto_type]
// Gets the post tags set on a specific post
pub fn post_tags_fragment() -> _ {
  let sel: SqlLiteral<Json> = diesel::dsl::sql::<diesel::sql_types::Json>("json_agg(tag.*)");
  post_tag::table
    .inner_join(tag::table)
    .select(sel)
    .filter(post_tag::post_id.eq(post::id))
    .filter(tag::deleted.eq(false))
    .single_value()
}

#[diesel::dsl::auto_type]
/// Gets the post tags available within a specific category
pub fn category_post_tags_fragment() -> _ {
  let sel: SqlLiteral<Json> = diesel::dsl::sql::<diesel::sql_types::Json>("json_agg(tag.*)");
  tag::table
    .select(sel)
    .filter(tag::category_id.eq(category::id))
    .filter(tag::deleted.eq(false))
    .single_value()
}

/// The select for the person1 alias.
pub fn person1_select() -> Person1AliasAllColumnsTuple {
  person1.fields(person::all_columns)
}

/// The select for the person2 alias.
pub fn person2_select() -> Person2AliasAllColumnsTuple {
  person2.fields(person::all_columns)
}

type IsSubscribedType =
  Eq<app_108jobs_db_schema_file::schema::category_actions::follow_state, Option<CategoryFollowerState>>;

pub fn filter_is_subscribed() -> IsSubscribedType {
  category_actions::follow_state.eq(Some(CategoryFollowerState::Accepted))
}

type IsNotUnlistedType =
  NotEq<app_108jobs_db_schema_file::schema::category::visibility, CategoryVisibility>;

#[diesel::dsl::auto_type]
pub fn filter_not_unlisted_or_is_subscribed() -> _ {
  let not_unlisted: IsNotUnlistedType = category::visibility.ne(CategoryVisibility::Unlisted);
  let is_subscribed: IsSubscribedType = filter_is_subscribed();
  not_unlisted.or(is_subscribed)
}

#[diesel::dsl::auto_type]
pub fn category_join() -> _ {
  // Use nullable().eq() since post.category_id is now nullable (for delivery posts)
  category::table.on(category::id.nullable().eq(post::category_id))
}

#[diesel::dsl::auto_type]
pub fn creator_home_instance_actions_join() -> _ {
  creator_home_instance_actions.on(
    creator_home_instance_actions
      .field(instance_actions::instance_id)
      .eq(person::instance_id)
      .and(
        creator_home_instance_actions
          .field(instance_actions::person_id)
          .eq(person::id),
      ),
  )
}
#[diesel::dsl::auto_type]
pub fn creator_category_instance_actions_join() -> _ {
  creator_category_instance_actions.on(
    creator_home_instance_actions
      .field(instance_actions::instance_id)
      .eq(category::instance_id)
      .and(
        creator_category_instance_actions
          .field(instance_actions::person_id)
          .eq(person::id),
      ),
  )
}

/// join with instance actions for local instance
///
/// Requires annotation for return type, see https://docs.diesel.rs/2.2.x/diesel/dsl/attr.auto_type.html#annotating-types
#[diesel::dsl::auto_type]
pub fn creator_local_instance_actions_join(local_instance_id: InstanceId) -> _ {
  creator_local_instance_actions.on(
    creator_local_instance_actions
      .field(instance_actions::instance_id)
      .eq(local_instance_id)
      .and(
        creator_local_instance_actions
          .field(instance_actions::person_id)
          .eq(person::id),
      ),
  )
}

/// Your instance actions for the category's instance.
#[diesel::dsl::auto_type]
pub fn my_instance_actions_category_join(my_person_id: Option<PersonId>) -> _ {
  instance_actions::table.on(
    instance_actions::instance_id
      .eq(category::instance_id)
      .and(instance_actions::person_id.nullable().eq(my_person_id)),
  )
}

/// Your instance actions for the person's instance.
#[diesel::dsl::auto_type]
pub fn my_instance_actions_person_join(my_person_id: Option<PersonId>) -> _ {
  instance_actions::table.on(
    instance_actions::instance_id
      .eq(person::instance_id)
      .and(instance_actions::person_id.nullable().eq(my_person_id)),
  )
}

#[diesel::dsl::auto_type]
pub fn image_details_join() -> _ {
  image_details::table.on(post::thumbnail_url.eq(image_details::link.nullable()))
}

#[diesel::dsl::auto_type]
pub fn my_category_actions_join(my_person_id: Option<PersonId>) -> _ {
  category_actions::table.on(
    category_actions::category_id
      .eq(category::id)
      .and(category_actions::person_id.nullable().eq(my_person_id)),
  )
}

#[diesel::dsl::auto_type]
pub fn my_post_actions_join(my_person_id: Option<PersonId>) -> _ {
  post_actions::table.on(
    post_actions::post_id
      .eq(post::id)
      .and(post_actions::person_id.nullable().eq(my_person_id)),
  )
}

#[diesel::dsl::auto_type]
pub fn my_comment_actions_join(my_person_id: Option<PersonId>) -> _ {
  comment_actions::table.on(
    comment_actions::comment_id
      .eq(comment::id)
      .and(comment_actions::person_id.nullable().eq(my_person_id)),
  )
}

#[diesel::dsl::auto_type]
pub fn my_person_actions_join(my_person_id: Option<PersonId>) -> _ {
  person_actions::table.on(
    person_actions::target_id
      .eq(person::id)
      .and(person_actions::person_id.nullable().eq(my_person_id)),
  )
}

#[diesel::dsl::auto_type]
pub fn my_local_user_admin_join(my_person_id: Option<PersonId>) -> _ {
  local_user::table.on(
    local_user::person_id
      .nullable()
      .eq(my_person_id)
      .and(local_user::admin.eq(true)),
  )
}

#[diesel::dsl::auto_type]
pub fn creator_category_actions_join() -> _ {
  creator_category_actions.on(
    creator_category_actions
      .field(category_actions::category_id)
      .eq(category::id)
      .and(
        creator_category_actions
          .field(category_actions::person_id)
          .eq(person::id),
      ),
  )
}
