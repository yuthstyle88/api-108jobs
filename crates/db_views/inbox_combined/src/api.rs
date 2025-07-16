use lemmy_db_schema::newtypes::{
  CommentReplyId,
  PersonCommentMentionId,
  PersonPostMentionId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A response containing a count of unread notifications.
#[serde(rename_all = "camelCase")]
pub struct GetUnreadCountResponse {
  pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The count of unread registration applications.
#[serde(rename_all = "camelCase")]
pub struct GetUnreadRegistrationApplicationCountResponse {
  pub registration_applications: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Mark a comment reply as read.
pub struct MarkCommentReplyAsRead {
  pub comment_reply_id: CommentReplyId,
  pub read: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Mark a person mention as read.
pub struct MarkPersonCommentMentionAsRead {
  pub person_comment_mention_id: PersonCommentMentionId,
  pub read: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Mark a person mention as read.
pub struct MarkPersonPostMentionAsRead {
  pub person_post_mention_id: PersonPostMentionId,
  pub read: bool,
}