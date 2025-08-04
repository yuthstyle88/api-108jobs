use actix_web::web::Data;
use either::Either::*;
use lemmy_api_utils::context::FastJobContext;
use lemmy_apub_objects::objects::{SearchableObjects, UserOrCommunity};
use lemmy_utils::error::FastJobResult;
use url::Url;

/// Converts search query to object id. The query can either be an URL, which will be treated as
/// ObjectId directly, or a webfinger identifier (@user@example.com or !community@example.com)
/// which gets resolved to an URL.
pub(crate) async fn search_query_to_object_id(
  mut query: String,
  context: &Data<FastJobContext>,
) -> FastJobResult<SearchableObjects> {
  todo!()
}

/// Converts a search query to an object id.  The query MUST bbe a URL which will bbe treated
/// as the ObjectId directly.  If the query is a webfinger identifier (@user@example.com or
/// !community@example.com) this method will return an error.
pub(crate) async fn search_query_to_object_id_local(
  query: &str,
  context: &Data<FastJobContext>,
) -> FastJobResult<SearchableObjects> {
 todo!()
}
