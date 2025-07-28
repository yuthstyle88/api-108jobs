use crate::check_report_reason;
use actix_web::web::{Data, Json};
use either::Either;
use lemmy_api_utils::{
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_comment_deleted_or_removed, slur_regex},
};
use lemmy_db_schema::{
  source::comment_report::{CommentReport, CommentReportForm},
  traits::Reportable,
};
use lemmy_db_views_comment::CommentView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_reports::{
  api::{CommentReportResponse, CreateCommentReport},
  CommentReportView,
};
use lemmy_utils::error::FastJobResult;

/// Creates a comment report and notifies the moderators of the community
pub async fn create_comment_report(
  data: Json<CreateCommentReport>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CommentReportResponse>> {
  let reason = data.reason.trim().to_string();
  let slur_regex = slur_regex(&context).await?;
  check_report_reason(&reason, &slur_regex)?;

  let person_id = local_user_view.person.id;
  let local_instance_id = local_user_view.person.instance_id;
  let comment_id = data.comment_id;
  let comment_view = CommentView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  // Don't allow creating reports for removed / deleted comments
  check_comment_deleted_or_removed(&comment_view.comment)?;

  let report_form = CommentReportForm {
    creator_id: person_id,
    comment_id,
    original_comment_text: comment_view.comment.content,
    reason,
    violates_instance_rules: data.violates_instance_rules.unwrap_or_default(),
  };

  let report = CommentReport::report(&mut context.pool(), &report_form).await?;

  let comment_report_view =
    CommentReportView::read(&mut context.pool(), report.id, person_id).await?;

  ActivityChannel::submit_activity(
    SendActivityData::CreateReport {
      actor: local_user_view.person,
      receiver: Either::Right(comment_view.community),
      reason: data.reason.clone(),
    },
    &context,
  )?;

  Ok(Json(CommentReportResponse {
    comment_report_view,
  }))
}
