use std::collections::HashMap;
use app_108jobs_db_schema::newtypes::{PersonId, PostId, RiderId};
use app_108jobs_db_schema::utils::{get_conn, DbPool};
use app_108jobs_db_schema_file::enums::PostKind;
use app_108jobs_utils::error::FastJobResult;

use app_108jobs_db_schema::source::delivery_details as dd_src;
use app_108jobs_db_schema::source::ride_session as ride_src;
use app_108jobs_db_views_rider::ride_session_view as rider_view;
use crate::api::PostItem;
use crate::PostView;

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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "view", rename_all = "camelCase")]
pub enum PostLogisticsView {
  Delivery(dd_src::DeliveryDetailsView),
  Ride(rider_view::RideSessionView),
}

impl LogisticsViewer {
  /// Convert to delivery details viewer
  pub fn to_delivery_viewer(self) -> dd_src::DeliveryDetailsViewer {
    match self {
      LogisticsViewer::Public => dd_src::DeliveryDetailsViewer::Public,
      LogisticsViewer::Employer(pid) => dd_src::DeliveryDetailsViewer::Employer(pid),
      LogisticsViewer::Rider(rid) => dd_src::DeliveryDetailsViewer::Rider(rid),
      LogisticsViewer::Admin => dd_src::DeliveryDetailsViewer::Admin,
    }
  }

  /// Convert to ride session viewer
  pub fn to_ride_viewer(self) -> rider_view::RideViewer {
    match self {
      LogisticsViewer::Public => rider_view::RideViewer::Public,
      LogisticsViewer::Employer(pid) => rider_view::RideViewer::Employer(pid),
      LogisticsViewer::Rider(rid) => rider_view::RideViewer::Rider(rid),
      LogisticsViewer::Admin => rider_view::RideViewer::Admin,
    }
  }
}

/// Logistics maps containing delivery_details and ride_sessions indexed by post_id
pub type LogisticsMaps = (
  HashMap<PostId, dd_src::DeliveryDetails>,
  HashMap<PostId, ride_src::RideSession>,
);

/// Fetch delivery_details and ride_sessions maps in batch from post IDs.
/// Reusable by both post views and search results.
pub async fn fetch_logistics_maps_by_ids(
  conn: &mut diesel_async::AsyncPgConnection,
  delivery_ids: &[PostId],
  ride_ids: &[PostId],
) -> FastJobResult<LogisticsMaps> {
  let delivery_map = if !delivery_ids.is_empty() {
    delivery_details::table
      .filter(delivery_details::post_id.eq_any(delivery_ids))
      .select(dd_src::DeliveryDetails::as_select())
      .load::<dd_src::DeliveryDetails>(conn)
      .await?
      .into_iter()
      .map(|dd| (dd.post_id, dd))
      .collect()
  } else {
    HashMap::new()
  };

  let ride_map = if !ride_ids.is_empty() {
    ride_session::table
      .filter(ride_session::post_id.eq_any(ride_ids))
      .select(ride_src::RideSession::as_select())
      .load::<ride_src::RideSession>(conn)
      .await?
      .into_iter()
      .map(|rs| (rs.post_id, rs))
      .collect()
  } else {
    HashMap::new()
  };

  Ok((delivery_map, ride_map))
}

/// Build logistics view for a single post from pre-fetched maps.
/// Reusable by both post views and search results.
pub fn build_logistics_from_maps(
  post_view: &PostView,
  maps: &LogisticsMaps,
  viewer: LogisticsViewer,
  is_admin: bool,
) -> Option<PostLogisticsView> {
  let (delivery_map, ride_map) = maps;
  match post_view.post.post_kind {
    PostKind::Delivery => delivery_map.get(&post_view.post.id).map(|full| {
      PostLogisticsView::Delivery(full.to_view(viewer.to_delivery_viewer(), post_view.creator.id, is_admin))
    }),
    PostKind::RideTaxi => ride_map.get(&post_view.post.id).map(|full| {
      PostLogisticsView::Ride(rider_view::project_ride_session(full, viewer.to_ride_viewer(), post_view.creator.id, is_admin))
    }),
    PostKind::Normal => None,
  }
}

/// Batch load logistics for a list of post views.
/// Optimized: Skips DB queries entirely if all posts are Normal kind.
/// For Delivery/RideTaxi posts, uses 2 batch queries instead of N+1.
pub async fn load_logistics_for_post_views(
  post_views: Vec<PostView>,
  pool: &mut DbPool<'_>,
  viewer: LogisticsViewer,
  is_admin: bool,
) -> FastJobResult<Vec<PostItem>> {
  // Early return if empty
  if post_views.is_empty() {
    return Ok(Vec::new());
  }

  // Check if any posts need logistics (before getting DB connection)
  let has_delivery = post_views.iter().any(|pv| pv.post.post_kind == PostKind::Delivery);
  let has_ride = post_views.iter().any(|pv| pv.post.post_kind == PostKind::RideTaxi);

  // If all posts are Normal, skip DB queries entirely
  if !has_delivery && !has_ride {
    return Ok(post_views
      .into_iter()
      .map(|post_view| PostItem { post_view, logistics: None })
      .collect());
  }

  // Get connection only if we need to fetch logistics
  let conn = &mut get_conn(pool).await?;

  // Collect post IDs by kind
  let delivery_ids: Vec<PostId> = post_views
    .iter()
    .filter(|pv| pv.post.post_kind == PostKind::Delivery)
    .map(|pv| pv.post.id)
    .collect();

  let ride_ids: Vec<PostId> = post_views
    .iter()
    .filter(|pv| pv.post.post_kind == PostKind::RideTaxi)
    .map(|pv| pv.post.id)
    .collect();

  // Fetch maps
  let maps = fetch_logistics_maps_by_ids(conn, &delivery_ids, &ride_ids).await?;

  // Build PostItems using the pre-fetched maps
  let items = post_views
    .into_iter()
    .map(|post_view| {
      let logistics = build_logistics_from_maps(&post_view, &maps, viewer, is_admin);
      PostItem { post_view, logistics }
    })
    .collect();

  Ok(items)
}

/// Load logistics for a single post (for cases like read/update single post).
/// Returns None immediately for Normal posts without DB query.
pub async fn load_post_logistics(
  pool: &mut DbPool<'_>,
  post_id: PostId,
  post_kind: PostKind,
  creator_person_id: PersonId,
  viewer: LogisticsViewer,
  is_admin: bool,
) -> FastJobResult<Option<PostLogisticsView>> {
  // Early return for Normal posts - no DB query needed
  if post_kind == PostKind::Normal {
    return Ok(None);
  }

  let conn = &mut get_conn(pool).await?;
  match post_kind {
    PostKind::Delivery => {
      let found: Option<dd_src::DeliveryDetails> = delivery_details::table
        .filter(delivery_details::post_id.eq(post_id))
        .select(dd_src::DeliveryDetails::as_select())
        .first::<dd_src::DeliveryDetails>(conn)
        .await
        .optional()?;

      Ok(found.map(|full| {
        PostLogisticsView::Delivery(full.to_view(viewer.to_delivery_viewer(), creator_person_id, is_admin))
      }))
    }
    PostKind::RideTaxi => {
      let found: Option<ride_src::RideSession> = ride_session::table
        .filter(ride_session::post_id.eq(post_id))
        .select(ride_src::RideSession::as_select())
        .first::<ride_src::RideSession>(conn)
        .await
        .optional()?;

      Ok(found.map(|full| {
        PostLogisticsView::Ride(rider_view::project_ride_session(&full, viewer.to_ride_viewer(), creator_person_id, is_admin))
      }))
    }
    PostKind::Normal => Ok(None),
  }
}
