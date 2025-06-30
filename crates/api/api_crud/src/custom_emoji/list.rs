use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_views_custom_emoji::{
  api::{ListCustomEmojis, ListCustomEmojisResponse},
  CustomEmojiView,
};
use lemmy_utils::error::FastJobError;

pub async fn list_custom_emojis(
  data: Query<ListCustomEmojis>,
  context: Data<FastJobContext>,
) -> Result<Json<ListCustomEmojisResponse>, FastJobError> {
  let custom_emojis = CustomEmojiView::list(&mut context.pool(), &data.category).await?;

  Ok(Json(ListCustomEmojisResponse { custom_emojis }))
}
