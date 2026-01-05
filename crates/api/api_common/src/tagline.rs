pub use app_108jobs_db_schema::{newtypes::TaglineId, source::tagline::Tagline};
pub use app_108jobs_db_views_site::api::{ListTaglines, ListTaglinesResponse, TaglineResponse};

pub mod aministration {
  pub use app_108jobs_db_views_site::api::{CreateTagline, DeleteTagline, UpdateTagline};
}
