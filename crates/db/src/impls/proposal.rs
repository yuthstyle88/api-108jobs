use crate::{
  diesel::NullableExpressionMethods,
  newtypes::{CategoryId, InstanceId, PersonId, ProposalId},
  schema::{category, post, proposal, proposal_actions},
  source::proposal::{
    Proposal,
    ProposalActions,
    ProposalInsertForm,
    ProposalLikeForm,
    ProposalSavedForm,
    ProposalUpdateForm,
  },
  traits::{Crud, Likeable, Saveable},
  utils::{functions::hot_rank, get_conn, uplete, validate_like, DbPool, DELETED_REPLACEMENT_TEXT},
};
use app_108jobs_core::{
  error::{FastJobErrorExt, FastJobErrorExt2, FastJobErrorType, FastJobResult},
  settings::structs::Settings,
};
use chrono::Utc;
use diesel::{
  dsl::insert_into,
  expression::SelectableHelper,
  update,
  ExpressionMethods,
  JoinOnDsl,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use url::Url;

impl Crud for Proposal {
  type InsertForm = ProposalInsertForm;
  type UpdateForm = ProposalUpdateForm;
  type IdType = ProposalId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(proposal::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateProposal)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    update(proposal::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateProposal)
  }
}

impl Proposal {
  pub async fn permadelete_for_creator(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    diesel::update(proposal::table.filter(proposal::creator_id.eq(creator_id)))
      .set((
        proposal::content.eq(DELETED_REPLACEMENT_TEXT),
        proposal::deleted.eq(true),
        proposal::updated_at.eq(Utc::now()),
      ))
      .get_results::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateProposal)
  }

  pub async fn update_removed_for_creator(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    removed: bool,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    update(proposal::table.filter(proposal::creator_id.eq(creator_id)))
      .set((
        proposal::removed.eq(removed),
        proposal::updated_at.eq(Utc::now()),
      ))
      .get_results::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateProposal)
  }

  /// Diesel can't update from join unfortunately, so you'll need to loop over these
  async fn creator_comment_ids_in_category(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    category_id: CategoryId,
  ) -> FastJobResult<Vec<ProposalId>> {
    let conn = &mut get_conn(pool).await?;

    proposal::table
      .inner_join(post::table)
      .filter(proposal::creator_id.eq(creator_id))
      .filter(post::category_id.eq(category_id))
      .select(proposal::id)
      .load::<ProposalId>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  /// Diesel can't update from join unfortunately, so you'll need to loop over these
  async fn creator_comment_ids_in_instance(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    instance_id: InstanceId,
  ) -> FastJobResult<Vec<ProposalId>> {
    let conn = &mut get_conn(pool).await?;
    // Use nullable().eq() to compare nullable post.category_id with category.id
    let category_join = category::table.on(category::id.nullable().eq(post::category_id));

    proposal::table
      .inner_join(post::table)
      .inner_join(category_join)
      .filter(proposal::creator_id.eq(creator_id))
      .filter(post::category_id.is_not_null()) // Only include comments on posts with categories
      .filter(category::instance_id.eq(instance_id))
      .select(proposal::id)
      .load::<ProposalId>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn update_removed_for_creator_and_category(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    category_id: CategoryId,
    removed: bool,
  ) -> FastJobResult<Vec<ProposalId>> {
    let comment_ids = Self::creator_comment_ids_in_category(pool, creator_id, category_id).await?;

    let conn = &mut get_conn(pool).await?;

    update(proposal::table)
      .filter(proposal::id.eq_any(comment_ids.clone()))
      .set((
        proposal::removed.eq(removed),
        proposal::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await?;

    Ok(comment_ids)
  }

  pub async fn update_removed_for_creator_and_instance(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    instance_id: InstanceId,
    removed: bool,
  ) -> FastJobResult<Vec<ProposalId>> {
    let comment_ids = Self::creator_comment_ids_in_instance(pool, creator_id, instance_id).await?;
    let conn = &mut get_conn(pool).await?;

    update(proposal::table)
      .filter(proposal::id.eq_any(comment_ids.clone()))
      .set((
        proposal::removed.eq(removed),
        proposal::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await?;
    Ok(comment_ids)
  }

  pub fn parent_comment_id(&self) -> Option<ProposalId> {
    let mut ltree_split: Vec<&str> = self.path.0.split('.').collect();
    ltree_split.remove(0); // The first is always 0
    if ltree_split.len() > 1 {
      let parent_comment_id = ltree_split.get(ltree_split.len() - 2);
      parent_comment_id.and_then(|p| p.parse::<i32>().map(ProposalId).ok())
    } else {
      None
    }
  }
  pub async fn update_hot_rank(
    pool: &mut DbPool<'_>,
    comment_id: ProposalId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    update(proposal::table.find(comment_id))
      .set(proposal::hot_rank.eq(hot_rank(proposal::score, proposal::published_at)))
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateProposal)
  }

  pub fn local_url(&self, settings: &Settings) -> FastJobResult<Url> {
    let domain = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!("{domain}/proposal/{}", self.id))?)
  }

  /// The proposal was created locally and sent back, indicating that the category accepted it
  pub async fn set_not_pending(&self, pool: &mut DbPool<'_>) -> FastJobResult<()> {
    if self.pending {
      let form = ProposalUpdateForm {
        pending: Some(false),
        ..Default::default()
      };
      Proposal::update(pool, self.id, &form).await?;
    }
    Ok(())
  }
}

impl Likeable for ProposalActions {
  type Form = ProposalLikeForm;
  type IdType = ProposalId;

  async fn like(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    validate_like(form.like_score).with_fastjob_type(FastJobErrorType::CouldntLikeProposal)?;

    insert_into(proposal_actions::table)
      .values(form)
      .on_conflict((proposal_actions::proposal_id, proposal_actions::person_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntLikeProposal)
  }
  async fn remove_like(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    comment_id: Self::IdType,
  ) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(proposal_actions::table.find((person_id, comment_id)))
      .set_null(proposal_actions::like_score)
      .set_null(proposal_actions::liked_at)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntLikeProposal)
  }

  async fn remove_all_likes(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
  ) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;

    uplete::new(proposal_actions::table.filter(proposal_actions::person_id.eq(creator_id)))
      .set_null(proposal_actions::like_score)
      .set_null(proposal_actions::liked_at)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateProposal)
  }

  async fn remove_likes_in_category(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    category_id: CategoryId,
  ) -> FastJobResult<uplete::Count> {
    let comment_ids =
      Proposal::creator_comment_ids_in_category(pool, creator_id, category_id).await?;

    let conn = &mut get_conn(pool).await?;

    uplete::new(
      proposal_actions::table.filter(proposal_actions::proposal_id.eq_any(comment_ids.clone())),
    )
    .set_null(proposal_actions::like_score)
    .set_null(proposal_actions::liked_at)
    .get_result(conn)
    .await
    .with_fastjob_type(FastJobErrorType::CouldntUpdateProposal)
  }
}

impl Saveable for ProposalActions {
  type Form = ProposalSavedForm;
  async fn save(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(proposal_actions::table)
      .values(form)
      .on_conflict((proposal_actions::proposal_id, proposal_actions::person_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntSaveProposal)
  }
  async fn unsave(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(proposal_actions::table.find((form.person_id, form.proposal_id)))
      .set_null(proposal_actions::saved_at)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntSaveProposal)
  }
}

impl ProposalActions {
  pub async fn read(
    pool: &mut DbPool<'_>,
    comment_id: ProposalId,
    person_id: PersonId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    proposal_actions::table
      .find((person_id, comment_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}
