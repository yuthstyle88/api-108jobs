use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::{context::FastJobContext, utils::is_admin};
use app_108jobs_db_schema::{
  source::{
    custom_emoji::{CustomEmoji, CustomEmojiUpdateForm},
    custom_emoji_keyword::{CustomEmojiKeyword, CustomEmojiKeywordInsertForm},
  },
  traits::Crud,
};
use app_108jobs_db_views_custom_emoji::{
  api::{CustomEmojiResponse, EditCustomEmojiRequest},
  CustomEmojiView,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_utils::error::FastJobResult;

pub async fn update_custom_emoji(
  data: Json<EditCustomEmojiRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CustomEmojiResponse>> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let emoji_form = CustomEmojiUpdateForm::new(
    data.clone().image_url.into(),
    data.alt_text.to_string(),
    data.category.to_string(),
  );
  let emoji = CustomEmoji::update(&mut context.pool(), data.id, &emoji_form).await?;
  CustomEmojiKeyword::delete(&mut context.pool(), data.id).await?;
  let mut keywords = vec![];
  for keyword in &data.keywords {
    let keyword_form =
      CustomEmojiKeywordInsertForm::new(emoji.id, keyword.to_lowercase().trim().to_string());
    keywords.push(keyword_form);
  }
  CustomEmojiKeyword::create(&mut context.pool(), keywords).await?;
  let view = CustomEmojiView::get(&mut context.pool(), emoji.id).await?;
  Ok(Json(CustomEmojiResponse { custom_emoji: view }))
}
