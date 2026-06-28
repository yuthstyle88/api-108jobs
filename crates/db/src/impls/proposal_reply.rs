use crate::{
  diesel::OptionalExtension,
  newtypes::{PersonId, ProposalId, ProposalReplyId},
  schema::proposal_reply,
  source::proposal_reply::{ProposalReply, ProposalReplyInsertForm, ProposalReplyUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use app_108jobs_core::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use diesel::{dsl::insert_into, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

impl Crud for ProposalReply {
  type InsertForm = ProposalReplyInsertForm;
  type UpdateForm = ProposalReplyUpdateForm;
  type IdType = ProposalReplyId;

  async fn create(
    pool: &mut DbPool<'_>,
    comment_reply_form: &Self::InsertForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    // since the return here isnt utilized, we dont need to do an update
    // but get_result doesn't return the existing row here
    insert_into(proposal_reply::table)
      .values(comment_reply_form)
      .on_conflict((proposal_reply::recipient_id, proposal_reply::comment_id))
      .do_update()
      .set(comment_reply_form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateProposalReply)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    proposal_reply_id: ProposalReplyId,
    comment_reply_form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(proposal_reply::table.find(proposal_reply_id))
      .set(comment_reply_form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateProposalReply)
  }
}

impl ProposalReply {
  pub async fn mark_all_as_read(
    pool: &mut DbPool<'_>,
    for_recipient_id: PersonId,
  ) -> FastJobResult<Vec<ProposalReply>> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(
      proposal_reply::table
        .filter(proposal_reply::recipient_id.eq(for_recipient_id))
        .filter(proposal_reply::read.eq(false)),
    )
    .set(proposal_reply::read.eq(true))
    .get_results::<Self>(conn)
    .await
    .with_fastjob_type(FastJobErrorType::CouldntMarkProposalReplyAsRead)
  }

  pub async fn read_by_comment(
    pool: &mut DbPool<'_>,
    for_comment_id: ProposalId,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    proposal_reply::table
      .filter(proposal_reply::comment_id.eq(for_comment_id))
      .first(conn)
      .await
      .optional()
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn read_by_comment_and_person(
    pool: &mut DbPool<'_>,
    for_comment_id: ProposalId,
    for_recipient_id: PersonId,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    proposal_reply::table
      .filter(proposal_reply::comment_id.eq(for_comment_id))
      .filter(proposal_reply::recipient_id.eq(for_recipient_id))
      .first(conn)
      .await
      .optional()
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}
