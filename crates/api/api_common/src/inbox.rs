pub use app_108jobs_db::{
  newtypes::{PersonPostMentionId, PersonProposalMentionId, ProposalReplyId},
  source::{
    person_post_mention::PersonPostMention, person_proposal_mention::PersonProposalMention,
    proposal_reply::ProposalReply,
  },
  InboxDataType,
};
pub use app_108jobs_db_views_inbox_combined::{
  api::{
    GetUnreadCountResponse, MarkPersonPostMentionAsRead, MarkPersonProposalMentionAsRead,
    MarkProposalReplyAsRead,
  },
  InboxCombinedView, ListInbox, ListInboxResponse, PersonPostMentionView,
  PersonProposalMentionView, ProposalReplyView,
};
