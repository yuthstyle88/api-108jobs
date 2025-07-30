use crate::newtypes::{LocalUserId, PostId, ProposalId};
use crate::source::proposal::{Proposal, ProposalInsertForm, ProposalUpdateForm};
use crate::traits::Crud;
use crate::utils::{get_conn, DbPool};
use diesel::associations::HasTable;
use diesel::{ ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::proposals::dsl::proposals;
use lemmy_db_schema_file::schema::proposals::{ deleted_at, id, post_id, user_id};
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

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

// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
// #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
// #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
// #[serde(rename_all = "camelCase")]
// pub struct ProposalView {
//     pub id: ProposalId,
//     pub description: String,
//     pub budget: f64,
//     pub working_days: i32,
//     pub brief_url: Option<String>,
//     pub user_id: LocalUserId,
//     pub post_id: PostId,
//     pub deleted_at: Option<chrono::DateTime<Utc>>,
//     pub created_at: chrono::DateTime<Utc>,
//     pub updated_at: chrono::DateTime<Utc>,
// }

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ListProposals {
  pub post_id: Option<i32>,
  pub user_id: Option<i32>,
  pub limit: Option<i64>,
  pub offset: Option<i64>,
  pub sort: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct ListProposalsResponse {
  pub proposals: Vec<Proposal>,
  pub total: i64,
  pub current_page: i64,
  pub total_pages: i64,
}

impl Proposal {
  pub async fn list(
    pool: &mut DbPool<'_>,
    data: &ListProposals,
  ) -> FastJobResult<ListProposalsResponse> {
    let conn = &mut get_conn(pool).await?;

    // Start with a base query for the *paginated results*
    // This query will receive the order, limit, and offset later
    let mut paginated_query_builder = proposals::table().into_boxed();

    // --- Filters ---
    paginated_query_builder = paginated_query_builder.filter(deleted_at.is_null());

    if let Some(pid) = data.post_id {
      paginated_query_builder = paginated_query_builder.filter(post_id.eq(pid));
    }
    if let Some(uid) = data.user_id {
      paginated_query_builder = paginated_query_builder.filter(user_id.eq(uid));
    }

    // --- Get Total Count ---
    // Create a *separate* query specifically for the total count,
    // applying the same filters as the main query.
    let mut count_query_builder = proposals::table().into_boxed(); // Start fresh for count
    count_query_builder = count_query_builder.filter(deleted_at.is_null());

    if let Some(pid) = data.post_id {
      count_query_builder = count_query_builder.filter(post_id.eq(pid));
    }
    if let Some(uid) = data.user_id {
      count_query_builder = count_query_builder.filter(user_id.eq(uid));
    }

    let total = count_query_builder
      .select(diesel::dsl::count_star()) // Explicitly select count_star
      .get_result::<i64>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;

    // --- Pagination ---
    let limit = data.limit.unwrap_or(10).max(1).min(50);
    let offset = data.offset.unwrap_or(0).max(0);

    paginated_query_builder = paginated_query_builder.limit(limit).offset(offset);

    // --- Execute Query for paginated results ---
    let proposals_data = paginated_query_builder
      .select(Proposal::as_select()) // Select all fields for the Proposal struct
      .load::<Proposal>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;

    // Calculate total_pages and current_page using concrete i64 values
    let total_pages = (total + limit - 1) / limit;
    let current_page = (offset / limit) + 1;

    Ok(ListProposalsResponse {
      proposals: proposals_data,
      total,
      current_page,
      total_pages,
    })
  }

  pub async fn find_by_user_and_job(
    pool: &mut DbPool<'_>,
    user_id_param: LocalUserId,
    post_id_param: PostId,
  ) -> Result<Option<Self>, diesel::result::Error> {
    let conn = &mut *get_conn(pool).await?;

    diesel_async::RunQueryDsl::first(
      proposals
        .filter(user_id.eq(user_id_param))
        .filter(post_id.eq(post_id_param)),
      conn,
    )
    .await
    .optional()
  }
}
