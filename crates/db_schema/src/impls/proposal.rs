use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::proposals::{created_at, deleted_at, id, post_id, user_id};
use lemmy_db_schema_file::schema::proposals::dsl::proposals;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use crate::newtypes::{ LocalUserId, PostId, ProposalId};
use crate::source::proposal::{Proposal, ProposalInsertForm, ProposalUpdateForm};
use crate::traits::Crud;
use crate::utils::{get_conn, DbPool};

impl Crud for Proposal {
    type InsertForm = ProposalInsertForm;
    type UpdateForm = ProposalUpdateForm;
    type IdType = ProposalId;

    async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(proposals)
            .values(form)
            .get_result::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntCreateEntry)
    }

    async fn read(pool: &mut DbPool<'_>, proposal_id: Self::IdType) -> FastJobResult<Self> {
        let conn = &mut get_conn(pool).await?;
        proposals
            .filter(id.eq(proposal_id))
            .filter(deleted_at.is_null())
            .first::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntFindProposal)
    }

    async fn update(
        pool: &mut DbPool<'_>,
        proposal_id: Self::IdType,
        form: &Self::UpdateForm,
    ) -> FastJobResult<Self> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(proposals)
            .filter(id.eq(proposal_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntUpdateProposal)
    }

    async fn delete(pool: &mut DbPool<'_>, proposal_id: Self::IdType) -> FastJobResult<usize> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(proposals)
            .filter(id.eq(proposal_id))
            .set(deleted_at.eq(chrono::Utc::now()))
            .execute(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntDeleteProposal)
    }
}

impl Proposal {
    pub async fn has_user_proposed(
        pool: &mut DbPool<'_>,
        user_id_param: LocalUserId,
        post_id_param: PostId,
    ) -> FastJobResult<bool> {
        let conn = &mut get_conn(pool).await?;

        let proposal_exists = proposals
            .filter(user_id.eq(user_id_param))
            .filter(post_id.eq(post_id_param))
            .filter(deleted_at.is_null())
            .select(id)
            .limit(1)
            .first::<ProposalId>(conn)
            .await
            .optional()?;
        Ok(proposal_exists.is_some())
    }

    pub async fn find_paginated(
        pool: &mut DbPool<'_>,
        page_num: u64,
        page_size: u64,
        post_id_input: Option<PostId>,
        freelance_id: Option<LocalUserId>,
    ) -> FastJobResult<(Vec<Self>, u64)> {
        let conn = &mut get_conn(pool).await?;

        let offset = (page_num - 1) * page_size;

        let mut query = proposals.into_boxed();

        if let Some(post_id_value) = post_id_input {
            query = query.filter(post_id.eq(post_id_value));
        }

        if let Some(freelancer_id) = freelance_id {
            query = query.filter(user_id.eq(freelancer_id));
        }

        query = query.filter(deleted_at.is_null());

        let proposals_list = query
            .order_by(created_at.desc())
            .limit(page_size as i64)
            .offset(offset as i64)
            .select(Self::as_select())
            .load::<Self>(conn)
            .await?;

        let mut count_query = proposals.into_boxed();

        if let Some(post_id_value) = post_id_input {
            count_query = count_query.filter(post_id.eq(post_id_value));
        }

        if let Some(freelancer_id) = freelance_id {
            count_query = count_query.filter(user_id.eq(freelancer_id));
        }

        count_query = count_query.filter(deleted_at.is_null());

        let total = count_query.count().get_result::<i64>(conn).await?;

        Ok((proposals_list, total as u64))
    }

    pub async fn find_by_user_and_job(
        pool: &mut DbPool<'_>,
        user_id_param: LocalUserId,
        post_id_param: PostId,
    ) -> FastJobResult<Option<Self>> {
        let conn = &mut get_conn(pool).await?;

        let result = proposals
            .filter(user_id.eq(user_id_param))
            .filter(post_id.eq(post_id_param))
            .filter(deleted_at.is_null())
            .first::<Self>(conn)
            .await
            .optional()?;

        Ok(result)
    }

    pub async fn list_by_job(
        pool: &mut DbPool<'_>,
        post_id_param: PostId,
    ) -> FastJobResult<Vec<Self>> {
        let conn = &mut get_conn(pool).await?;

        proposals
            .filter(post_id.eq(post_id_param))
            .filter(deleted_at.is_null())
            .order_by(created_at.desc())
            .load::<Self>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntListProposals)
    }
}