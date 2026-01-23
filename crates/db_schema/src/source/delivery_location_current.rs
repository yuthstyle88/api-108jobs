use crate::newtypes::{PostId, RiderId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
use app_108jobs_db_schema_file::schema::delivery_location_current;

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = delivery_location_current))]
#[cfg_attr(feature = "full", diesel(primary_key(post_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Current delivery location of a rider for a post.
pub struct DeliveryLocationCurrent {
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

  /// Last update time
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, Serialize, Deserialize))]
#[cfg_attr(feature = "full", diesel(table_name = delivery_location_current))]
pub struct DeliveryLocationCurrentInsertForm {
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
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset, Serialize, Deserialize))]
#[cfg_attr(feature = "full", diesel(table_name = delivery_location_current))]
pub struct DeliveryLocationCurrentUpdateForm {
  pub lat: Option<f64>,
  pub lng: Option<f64>,
  pub heading: Option<Option<f64>>,
  pub speed_kmh: Option<Option<f64>>,
  pub accuracy_m: Option<Option<f64>>,
  pub updated_at: Option<DateTime<Utc>>,
}
