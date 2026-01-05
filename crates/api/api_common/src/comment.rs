pub use app_108jobs_db_schema::{
  newtypes::CommentId,
  source::comment::{Comment, CommentActions},
};
pub use app_108jobs_db_views_comment::{
  api::{CommentResponse, GetComment, GetComments, GetCommentsResponse, GetCommentsSlimResponse},
  CommentSlimView,
  CommentView,
};

pub mod actions {
  pub use app_108jobs_db_views_comment::api::{
    CreateComment,
    CreateCommentLike,
    DeleteComment,
    EditComment,
    SaveComment,
  };

  pub mod moderation {
    pub use app_108jobs_db_views_comment::api::{
      DistinguishComment,
      ListCommentLikes,
      ListCommentLikesResponse,
      PurgeComment,
      RemoveComment,
    };
  }
}
