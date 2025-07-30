use actix_web::HttpRequest;
use actix_web::web::{Data, Json};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::proposal::{Proposal, ProposalInsertForm};
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobError, FastJobErrorType, FastJobResult};
use log::debug;
use lemmy_db_views_proposal::{CreateProposalRequest, CreateProposalResponse};

pub async fn create_proposal(
    data: Json<CreateProposalRequest>,
    _req: HttpRequest,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<CreateProposalResponse>> {
    let local_user_view = LocalUserView::read(&mut context.pool(), local_user_view.local_user.id)
        .await
        .map_err(|e| FastJobError::from(FastJobErrorType::DatabaseError(format!("Failed to read user: {}", e))))?;

    let job_post = JobPost::find_by_id(&mut context.pool(), data.job_post_id).await?;

    if !job_post.is_open {
        return Err(FastJobError::from(FastJobErrorType::ValidationError(
            "The job post must be open to send proposal".to_string(),
        )));
    }

    if data.budget <= 0.0 {
        return Err(FastJobError::from(FastJobErrorType::ValidationError(
            "Budget must be greater than 0".to_string(),
        )));
    }
    if data.working_days <= 0 {
        return Err(FastJobError::from(FastJobErrorType::ValidationError(
            "Working days must be greater than 0".to_string(),
        )));
    }
    if data.description.trim().is_empty() {
        return Err(FastJobError::from(FastJobErrorType::ValidationError(
            "Description cannot be empty".to_string(),
        )));
    }

    let existing_proposal = Proposal::find_by_user_and_job(
        &mut context.pool(),
        local_user_view.local_user.id,
        data.job_post_id,
    )
        .await?;

    if let Some(proposal) = existing_proposal {
        debug!("User {} tried to submit duplicate proposal for job {}", local_user_view.local_user.id, data.job_post_id);
        return Err(FastJobError::from(FastJobErrorType::ValidationError(
            format!(
                "You have already submitted proposal ID {:?} for this job post",
                proposal.id
            ),
        )));
    }

    let mut conn = context.pool().get().await?;
    let inserted_proposal = diesel::PgConnection::transaction(&mut conn, |conn| {
        let proposal_form = ProposalInsertForm {
            description: data.description.clone(),
            budget: data.budget,
            working_days: data.working_days,
            brief_url: data.brief_url.clone(),
            service_id: data.service_id,
            user_id: local_user_view.local_user.id,
            job_id: data.job_post_id,
        };

        Proposal::create(conn, &proposal_form)
    }).await?;

    debug!("Created proposal {} for user {} and job {}", inserted_proposal.id, local_user_view.local_user.id, data.job_post_id);

    let response = CreateProposalResponse {
        id: inserted_proposal.id,
        description: inserted_proposal.description,
        budget: inserted_proposal.budget,
        working_days: inserted_proposal.working_days,
        brief_url: inserted_proposal.brief_url,
        service_id: inserted_proposal.service_id,
        user_id: inserted_proposal.user_id,
        job_post_id: inserted_proposal.job_id,
        created_at: inserted_proposal.created_at,  // Thêm timestamps
        updated_at: inserted_proposal.updated_at,
    };

    Ok(Json(response))
}