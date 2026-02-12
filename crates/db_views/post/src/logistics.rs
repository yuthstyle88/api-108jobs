use app_108jobs_db_schema::newtypes::{PersonId, PostId, RiderId};
use app_108jobs_db_schema::utils::{get_conn, DbPool};
use app_108jobs_db_schema_file::enums::PostKind;
use app_108jobs_utils::error::FastJobResult;

use app_108jobs_db_schema::source::delivery_details as dd_src;
use app_108jobs_db_schema::source::ride_session as ride_src;
use app_108jobs_db_views_rider::ride_session_view as rider_view;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use app_108jobs_db_schema_file::schema::{delivery_details, ride_session};

#[derive(Debug, Clone, Copy)]
pub enum LogisticsViewer {
  Public,
  Employer(PersonId),
  Rider(RiderId),
  Admin,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "view", rename_all = "camelCase")]
pub enum PostLogisticsView {
  Delivery(dd_src::DeliveryDetailsView),
  Ride(rider_view::RideSessionView),
}

/// Load and project logistics for a post according to its kind.
pub async fn load_post_logistics(
  pool: &mut DbPool<'_>,
  post_id: PostId,
  post_kind: PostKind,
  creator_person_id: PersonId,
  viewer: LogisticsViewer,
  is_admin: bool,
) -> FastJobResult<Option<PostLogisticsView>> {
  let conn = &mut get_conn(pool).await?;
  match post_kind {
    PostKind::Delivery => {
      // Fetch delivery_details by post_id
      let found: Option<dd_src::DeliveryDetails> = delivery_details::table
        .filter(delivery_details::post_id.eq(post_id))
        .select(dd_src::DeliveryDetails::as_select())
        .first::<dd_src::DeliveryDetails>(conn)
        .await
        .optional()?;

      let view = found.map(|full| {
        let dd_viewer = match viewer {
          LogisticsViewer::Public => dd_src::DeliveryDetailsViewer::Public,
          LogisticsViewer::Employer(pid) => dd_src::DeliveryDetailsViewer::Employer(pid),
          LogisticsViewer::Rider(rid) => dd_src::DeliveryDetailsViewer::Rider(rid),
          LogisticsViewer::Admin => dd_src::DeliveryDetailsViewer::Admin,
        };
        PostLogisticsView::Delivery(full.to_view(dd_viewer, creator_person_id, is_admin))
      });
      Ok(view)
    }
    PostKind::RideTaxi => {
      // Fetch ride_session by post_id
      let found: Option<ride_src::RideSession> = ride_session::table
        .filter(ride_session::post_id.eq(post_id))
        .select(ride_src::RideSession::as_select())
        .first::<ride_src::RideSession>(conn)
        .await
        .optional()?;

      let view = found.map(|full| {
        let r_viewer = match viewer {
          LogisticsViewer::Public => rider_view::RideViewer::Public,
          LogisticsViewer::Employer(pid) => rider_view::RideViewer::Employer(pid),
          LogisticsViewer::Rider(rid) => rider_view::RideViewer::Rider(rid),
          LogisticsViewer::Admin => rider_view::RideViewer::Admin,
        };
        PostLogisticsView::Ride(rider_view::project_ride_session(&full, r_viewer, creator_person_id, is_admin))
      });
      Ok(view)
    }
    PostKind::Normal => Ok(None),
  }
}
