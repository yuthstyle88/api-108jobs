use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::{
  source::person_post_mention::{PersonPostMention, PersonPostMentionUpdateForm},
  traits::Crud,
};
use app_108jobs_db_views_inbox_combined::api::MarkPersonPostMentionAsRead;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};

pub async fn mark_post_mention_as_read(
  data: Json<MarkPersonPostMentionAsRead>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let person_post_mention_id = data.person_post_mention_id;
  let read_person_post_mention =
    PersonPostMention::read(&mut context.pool(), person_post_mention_id).await?;

  if local_user_view.person.id != read_person_post_mention.recipient_id {
    Err(FastJobErrorType::CouldntUpdatePost)?
  }

  let person_post_mention_id = read_person_post_mention.id;
  let read = Some(data.read);
  PersonPostMention::update(
    &mut context.pool(),
    person_post_mention_id,
    &PersonPostMentionUpdateForm { read },
  )
  .await?;

  Ok(Json(SuccessResponse::default()))
}
