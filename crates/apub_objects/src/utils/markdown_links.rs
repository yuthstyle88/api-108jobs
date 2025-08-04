use actix_web::web::Data;
use either::Either::*;
use lemmy_api_utils::context::FastJobContext;
use lemmy_utils::utils::markdown::image_links::{markdown_find_links, markdown_handle_title};
use url::Url;

pub async fn markdown_rewrite_remote_links_opt(
  src: Option<String>,
  context: &Data<FastJobContext>,
) -> Option<String> {
  match src {
    Some(t) => Some(markdown_rewrite_remote_links(t, context).await),
    None => None,
  }
}

/// Goes through all remote markdown links and attempts to resolve them as Activitypub objects.
/// If successful, the link is rewritten to a local link, so it can be viewed without leaving the
/// local instance.
///
/// As it relies on ObjectId::dereference, it can only be used for incoming federated objects, not
/// for the API.
pub async fn markdown_rewrite_remote_links(
  mut src: String,
  context: &Data<FastJobContext>,
) -> String {
  let links_offsets = markdown_find_links(&src);

  // Go through the collected links in reverse order
  for (start, end) in links_offsets.into_iter().rev() {
    let (url, extra) = markdown_handle_title(&src, start, end);

    if let Some(local_url) = to_local_url(url).await {
      let mut local_url = local_url.to_string();
      // restore title
      if let Some(extra) = extra {
        local_url.push(' ');
        local_url.push_str(extra);
      }
      src.replace_range(start..end, local_url.as_str());
    }
  }

  src
}

pub(crate) async fn to_local_url(url: &str) -> Option<Url> {
  Some(Url::parse(url).ok()?)
}
