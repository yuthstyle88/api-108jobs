use actix_web::web::Json;
use activitypub_federation::config::Data;
use chrono::Utc;
use lemmy_api_utils::{
  context::FastJobContext,
  utils::{get_url_blocklist, is_admin, process_markdown, slur_regex},
};
use lemmy_db_schema::{
  source::tagline::{Tagline, TaglineUpdateForm},
  traits::Crud,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::{TaglineResponse, UpdateTagline};
use lemmy_utils::error::FastJobError;

pub async fn update_tagline(
  data: Json<UpdateTagline>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> Result<Json<TaglineResponse>, FastJobError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  let content = process_markdown(&data.content, &slur_regex, &url_blocklist, &context).await?;

  let tagline_form = TaglineUpdateForm {
    content,
    updated_at: Some(Some(Utc::now())),
  };

  let tagline = Tagline::update(&mut context.pool(), data.id, &tagline_form).await?;

  Ok(Json(TaglineResponse { tagline }))
}
