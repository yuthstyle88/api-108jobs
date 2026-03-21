use crate::newtypes::{TripLocationHistoryId, PostId, RiderId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
use app_108jobs_db_schema_file::schema::trip_location_history;

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = trip_location_history))]
#[cfg_attr(feature = "full", diesel(primary_key(id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Historical trip location records (append-only, shared by delivery and ride taxi).
pub struct TripLocationHistory {
  pub id: TripLocationHistoryId,

  pub post_id: PostId,
  pub rider_id: RiderId,

  /// Latitude in decimal degrees
  pub lat: f64,

  /// Longitude in decimal degrees
  pub lng: f64,

  /// Heading in degrees
  pub heading: Option<f64>,

  /// Speed in km/h
  pub speed_kmh: Option<f64>,

  /// Accuracy in meters
  pub accuracy_m: Option<f64>,

  /// Time when this location was recorded
  pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, Serialize, Deserialize))]
#[cfg_attr(feature = "full", diesel(table_name = trip_location_history))]
pub struct TripLocationHistoryInsertForm {
  pub post_id: PostId,
  pub rider_id: RiderId,
  pub lat: f64,
  pub lng: f64,

  #[new(default)]
  pub heading: Option<f64>,

  #[new(default)]
  pub speed_kmh: Option<f64>,

  #[new(default)]
  pub accuracy_m: Option<f64>,

  #[new(value = "Utc::now()")]
  pub recorded_at: DateTime<Utc>,
}
