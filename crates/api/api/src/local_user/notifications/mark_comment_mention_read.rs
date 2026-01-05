use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::{
  source::person_comment_mention::{PersonCommentMention, PersonCommentMentionUpdateForm},
  traits::Crud,
};
use app_108jobs_db_views_inbox_combined::api::MarkPersonCommentMentionAsRead;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};

pub async fn mark_comment_mention_as_read(
  data: Json<MarkPersonCommentMentionAsRead>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let person_comment_mention_id = data.person_comment_mention_id;
  let read_person_comment_mention =
    PersonCommentMention::read(&mut context.pool(), person_comment_mention_id).await?;

  if local_user_view.person.id != read_person_comment_mention.recipient_id {
    Err(FastJobErrorType::CouldntUpdateComment)?
  }

  let person_comment_mention_id = read_person_comment_mention.id;
  let read = Some(data.read);
  PersonCommentMention::update(
    &mut context.pool(),
    person_comment_mention_id,
    &PersonCommentMentionUpdateForm { read },
  )
  .await?;

  Ok(Json(SuccessResponse::default()))
}
