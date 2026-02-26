use app_108jobs_db_schema::newtypes::{PersonId, RiderId};
use app_108jobs_db_schema::source::ride_session::RideSession;
use app_108jobs_db_schema_file::enums::{DeliveryStatus, PaymentMethod};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RideViewer {
  Public,
  Employer(PersonId),
  Rider(RiderId),
  Admin,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct RidePublic {
  pub id: app_108jobs_db_schema::newtypes::RideSessionId,
  pub post_id: app_108jobs_db_schema::newtypes::PostId,
  pub pickup_address: String,
  pub dropoff_address: String,
  pub status: DeliveryStatus,
  pub rider_id: Option<RiderId>,
  pub rider_assigned_at: Option<DateTime<Utc>>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct RidePrivate {
  pub id: app_108jobs_db_schema::newtypes::RideSessionId,
  pub post_id: app_108jobs_db_schema::newtypes::PostId,
  pub employer_id: app_108jobs_db_schema::newtypes::LocalUserId,
  pub rider_id: Option<RiderId>,
  pub pricing_config_id: Option<app_108jobs_db_schema::newtypes::PricingConfigId>,
  pub pickup_address: String,
  pub pickup_lat: Option<f64>,
  pub pickup_lng: Option<f64>,
  pub dropoff_address: String,
  pub dropoff_lat: Option<f64>,
  pub dropoff_lng: Option<f64>,
  pub pickup_note: Option<String>,
  // Passenger contact info - only visible to authorized parties
  pub passenger_name: Option<String>,
  pub passenger_phone: Option<String>,
  pub payment_method: PaymentMethod,
  pub payment_status: String,
  pub status: DeliveryStatus,
  pub requested_at: DateTime<Utc>,
  pub rider_assigned_at: Option<DateTime<Utc>>,
  pub rider_confirmed_at: Option<DateTime<Utc>>,
  pub arrived_at_pickup_at: Option<DateTime<Utc>>,
  pub ride_started_at: Option<DateTime<Utc>>,
  pub ride_completed_at: Option<DateTime<Utc>>,
  pub current_price_coin: i32,
  pub total_distance_km: Option<f64>,
  pub total_duration_minutes: Option<i32>,
  pub final_price_coin: Option<i32>,
  pub base_fare_applied_coin: Option<i32>,
  pub time_charge_applied_coin: Option<i32>,
  pub distance_charge_applied_coin: Option<i32>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum RideSessionView {
  Public(RidePublic),
  Private(RidePrivate),
}

pub fn project_ride_session(
  session: &RideSession,
  viewer: RideViewer,
  creator_person_id: PersonId,
  is_admin: bool,
) -> RideSessionView {
  let employer_matches = matches!(viewer, RideViewer::Employer(pid) if pid == creator_person_id);
  let rider_matches = matches!(viewer, RideViewer::Rider(rid) if Some(rid) == session.rider_id);

  if is_admin || employer_matches || rider_matches {
    RideSessionView::Private(RidePrivate {
      id: session.id,
      post_id: session.post_id,
      employer_id: session.employer_id,
      rider_id: session.rider_id,
      pricing_config_id: session.pricing_config_id,
      pickup_address: session.pickup_address.clone(),
      pickup_lat: session.pickup_lat,
      pickup_lng: session.pickup_lng,
      dropoff_address: session.dropoff_address.clone(),
      dropoff_lat: session.dropoff_lat,
      dropoff_lng: session.dropoff_lng,
      pickup_note: session.pickup_note.clone(),
      passenger_name: session.passenger_name.clone(),
      passenger_phone: session.passenger_phone.clone(),
      payment_method: session.payment_method,
      payment_status: session.payment_status.clone(),
      status: session.status,
      requested_at: session.requested_at,
      rider_assigned_at: session.rider_assigned_at,
      rider_confirmed_at: session.rider_confirmed_at,
      arrived_at_pickup_at: session.arrived_at_pickup_at,
      ride_started_at: session.ride_started_at,
      ride_completed_at: session.ride_completed_at,
      current_price_coin: session.current_price_coin,
      total_distance_km: session.total_distance_km,
      total_duration_minutes: session.total_duration_minutes,
      final_price_coin: session.final_price_coin,
      base_fare_applied_coin: session.base_fare_applied_coin,
      time_charge_applied_coin: session.time_charge_applied_coin,
      distance_charge_applied_coin: session.distance_charge_applied_coin,
      created_at: session.created_at,
      updated_at: session.updated_at,
    })
  } else {
    RideSessionView::Public(RidePublic {
      id: session.id,
      post_id: session.post_id,
      pickup_address: session.pickup_address.clone(),
      dropoff_address: session.dropoff_address.clone(),
      status: session.status,
      rider_id: session.rider_id,
      rider_assigned_at: session.rider_assigned_at,
    })
  }
}
