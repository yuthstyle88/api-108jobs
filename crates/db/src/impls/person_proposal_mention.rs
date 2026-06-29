use crate::{
  diesel::OptionalExtension,
  newtypes::{PersonId, PersonProposalMentionId, ProposalId},
  schema::person_proposal_mention,
  source::person_proposal_mention::{
    PersonProposalMention, PersonProposalMentionInsertForm, PersonProposalMentionUpdateForm,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use app_108jobs_core::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use diesel::{dsl::insert_into, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

impl Crud for PersonProposalMention {
  type InsertForm = PersonProposalMentionInsertForm;
  type UpdateForm = PersonProposalMentionUpdateForm;
  type IdType = PersonProposalMentionId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    // since the return here isnt utilized, we dont need to do an update
    // but get_result doesn't return the existing row here
    insert_into(person_proposal_mention::table)
      .values(form)
      .on_conflict((
        person_proposal_mention::recipient_id,
        person_proposal_mention::comment_id,
      ))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreatePersonProposalMention)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    person_proposal_mention_id: PersonProposalMentionId,
    person_comment_mention_form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(person_proposal_mention::table.find(person_proposal_mention_id))
      .set(person_comment_mention_form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdatePersonProposalMention)
  }
}

impl PersonProposalMention {
  pub async fn mark_all_as_read(
    pool: &mut DbPool<'_>,
    for_recipient_id: PersonId,
  ) -> FastJobResult<Vec<PersonProposalMention>> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(
      person_proposal_mention::table
        .filter(person_proposal_mention::recipient_id.eq(for_recipient_id))
        .filter(person_proposal_mention::read.eq(false)),
    )
    .set(person_proposal_mention::read.eq(true))
    .get_results::<Self>(conn)
    .await
    .with_fastjob_type(FastJobErrorType::CouldntUpdatePersonProposalMention)
  }

  pub async fn read_by_comment_and_person(
    pool: &mut DbPool<'_>,
    for_comment_id: ProposalId,
    for_recipient_id: PersonId,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    person_proposal_mention::table
      .filter(person_proposal_mention::comment_id.eq(for_comment_id))
      .filter(person_proposal_mention::recipient_id.eq(for_recipient_id))
      .first(conn)
      .await
      .optional()
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}
