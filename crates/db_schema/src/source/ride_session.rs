use crate::newtypes::{LocalUserId, PostId, PricingConfigId, RiderId, RideSessionId};
use app_108jobs_db_schema_file::enums::{DeliveryStatus, PaymentMethod};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use app_108jobs_db_schema_file::schema::ride_session;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable)
)]
#[cfg_attr(feature = "full", diesel(table_name = ride_session))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct RideSession {
  pub id: RideSessionId,
  pub post_id: PostId,
  pub rider_id: Option<RiderId>,  // NULL until a rider accepts the ride
  pub employer_id: LocalUserId,
  pub pricing_config_id: Option<PricingConfigId>,

  // Route & Payment
  pub pickup_address: String,
  pub pickup_lat: Option<f64>,
  pub pickup_lng: Option<f64>,
  pub dropoff_address: String,
  pub dropoff_lat: Option<f64>,
  pub dropoff_lng: Option<f64>,
  pub pickup_note: Option<String>,

  // Passenger contact info
  pub passenger_name: Option<String>,
  pub passenger_phone: Option<String>,

  // Payment method
  #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
  pub payment_method: PaymentMethod,
  pub payment_status: String,

  // Session state - uses DeliveryStatus for both taxi and cargo
  #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
  pub status: DeliveryStatus,

  // Timestamps
  pub requested_at: DateTime<Utc>,
  pub rider_assigned_at: Option<DateTime<Utc>>,
  pub rider_confirmed_at: Option<DateTime<Utc>>,
  pub arrived_at_pickup_at: Option<DateTime<Utc>>,
  pub ride_started_at: Option<DateTime<Utc>>,
  pub ride_completed_at: Option<DateTime<Utc>>,

  // Real-time meter data (stored in Coins)
  pub current_price_coin: i32,

  // Final calculated values
  pub total_distance_km: Option<f64>,
  pub total_duration_minutes: Option<i32>,
  pub final_price_coin: Option<i32>,

  // Pricing breakdown
  pub base_fare_applied_coin: Option<i32>,
  pub time_charge_applied_coin: Option<i32>,
  pub distance_charge_applied_coin: Option<i32>,

  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(
  feature = "full",
  derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
#[cfg_attr(feature = "full", diesel(table_name = ride_session))]
pub struct RideSessionInsertForm {
  pub post_id: PostId,
  pub rider_id: Option<RiderId>,
  pub employer_id: LocalUserId,
  pub pricing_config_id: Option<PricingConfigId>,
  pub pickup_address: String,
  pub pickup_lat: Option<f64>,
  pub pickup_lng: Option<f64>,
  pub dropoff_address: String,
  pub dropoff_lat: Option<f64>,
  pub dropoff_lng: Option<f64>,
  pub pickup_note: Option<String>,
  pub passenger_name: Option<String>,
  pub passenger_phone: Option<String>,
  pub payment_method: PaymentMethod,
  pub payment_status: Option<String>,
  pub status: Option<DeliveryStatus>,
  pub requested_at: Option<DateTime<Utc>>,
  pub current_price_coin: Option<i32>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset, Serialize, Deserialize))]
#[cfg_attr(feature = "full", diesel(table_name = ride_session))]
pub struct RideSessionUpdateForm {
  pub rider_id: Option<Option<RiderId>>,  // Nullable column, so Option<Option<>>
  pub pricing_config_id: Option<Option<PricingConfigId>>,
  pub pickup_lat: Option<Option<f64>>,
  pub pickup_lng: Option<Option<f64>>,
  pub dropoff_lat: Option<Option<f64>>,
  pub dropoff_lng: Option<Option<f64>>,
  pub pickup_note: Option<Option<String>>,
  pub passenger_name: Option<Option<String>>,
  pub passenger_phone: Option<Option<String>>,
  pub payment_status: Option<String>,
  pub status: Option<DeliveryStatus>,
  pub rider_assigned_at: Option<Option<DateTime<Utc>>>,
  pub rider_confirmed_at: Option<Option<DateTime<Utc>>>,
  pub arrived_at_pickup_at: Option<Option<DateTime<Utc>>>,
  pub ride_started_at: Option<Option<DateTime<Utc>>>,
  pub ride_completed_at: Option<Option<DateTime<Utc>>>,
  pub current_price_coin: Option<i32>,
  pub total_distance_km: Option<Option<f64>>,
  pub total_duration_minutes: Option<Option<i32>>,
  pub final_price_coin: Option<Option<i32>>,
  pub base_fare_applied_coin: Option<Option<i32>>,
  pub time_charge_applied_coin: Option<Option<i32>>,
  pub distance_charge_applied_coin: Option<Option<i32>>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
}
