use actix_web::web::Json;
use activitypub_federation::config::Data;
use lemmy_api_utils::{
  context::FastJobContext,
  utils::{get_url_blocklist, is_admin, process_markdown, slur_regex},
};
use lemmy_db_schema::{
  source::tagline::{Tagline, TaglineInsertForm},
  traits::Crud,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::{CreateTagline, TaglineResponse};
use lemmy_utils::error::FastJobError;

pub async fn create_tagline(
  data: Json<CreateTagline>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> Result<Json<TaglineResponse>, FastJobError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  let content = process_markdown(&data.content, &slur_regex, &url_blocklist, &context).await?;

  let tagline_form = TaglineInsertForm { content };

  let tagline = Tagline::create(&mut context.pool(), &tagline_form).await?;

  Ok(Json(TaglineResponse { tagline }))
}
