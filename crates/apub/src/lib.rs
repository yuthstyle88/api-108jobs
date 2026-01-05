use app_108jobs_db_schema::utils::ActualDbPool;
pub mod api;
pub mod fetcher;

/// Maximum number of outgoing HTTP requests to fetch a single object. Needs to be high enough
/// to fetch a new category with posts, moderators and featured posts.
pub const FEDERATION_HTTP_FETCH_LIMIT: u32 = 100;

#[derive(Clone)]
pub struct VerifyUrlData(pub ActualDbPool);
