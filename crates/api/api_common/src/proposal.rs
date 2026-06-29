pub use app_108jobs_db::{
  newtypes::ProposalId,
  source::proposal::{Proposal, ProposalActions},
};
pub use app_108jobs_db_views_proposal::{
  api::{GetComment, GetComments, GetCommentsResponse, GetCommentsSlimResponse, ProposalResponse},
  ProposalSlimView,
  ProposalView,
};

pub mod actions {
  pub use app_108jobs_db_views_proposal::api::{
    CreateComment,
    CreateCommentLike,
    DeleteComment,
    EditComment,
    SaveComment,
  };

  pub mod moderation {
    pub use app_108jobs_db_views_proposal::api::{
      DistinguishComment,
      ListCommentLikes,
      ListCommentLikesResponse,
      PurgeComment,
      RemoveComment,
    };
  }
}
