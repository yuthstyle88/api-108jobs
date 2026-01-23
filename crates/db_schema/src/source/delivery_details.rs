use crate::newtypes::{DeliveryDetailsId, PostId};
use crate::utils;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  app_108jobs_db_schema_file::{enums::DeliveryStatus, schema::delivery_details},
  diesel::prelude::*,
  diesel_async::RunQueryDsl,
};
use app_108jobs_db_schema_file::enums::VehicleType;

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(
  feature = "full",
  derive(diesel::Queryable, diesel::Selectable, diesel::Identifiable)
)]
#[cfg_attr(feature = "full", diesel(table_name = delivery_details))]
#[cfg_attr(feature = "full", diesel(primary_key(id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct DeliveryDetails {
  pub id: DeliveryDetailsId,
  pub post_id: PostId,
  pub pickup_address: String,
  pub pickup_lat: Option<f64>,
  pub pickup_lng: Option<f64>,
  pub dropoff_address: String,
  pub dropoff_lat: Option<f64>,
  pub dropoff_lng: Option<f64>,
  pub package_description: Option<String>,
  pub package_weight_kg: Option<f64>,
  pub package_size: Option<String>,
  pub fragile: bool,
  pub requires_signature: bool,
  pub vehicle_required: Option<VehicleType>,
  pub latest_pickup_at: Option<DateTime<Utc>>,
  pub latest_dropoff_at: Option<DateTime<Utc>>,
  pub sender_name: Option<String>,
  pub sender_phone: Option<String>,
  pub receiver_name: Option<String>,
  pub receiver_phone: Option<String>,
  pub cash_on_delivery: bool,
  pub cod_amount: Option<f64>,
  pub status: DeliveryStatus,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, derive_new::new, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = delivery_details))]
pub struct DeliveryDetailsInsertForm {
  pub post_id: PostId,
  // Locations
  pub pickup_address: String,
  #[new(default)]
  pub pickup_lat: Option<f64>,
  #[new(default)]
  pub pickup_lng: Option<f64>,
  pub dropoff_address: String,
  #[new(default)]
  pub dropoff_lat: Option<f64>,
  #[new(default)]
  pub dropoff_lng: Option<f64>,

  // Package
  #[new(default)]
  pub package_description: Option<String>,
  #[new(default)]
  pub package_weight_kg: Option<f64>,
  #[new(default)]
  pub package_size: Option<String>,
  #[new(default)]
  pub fragile: Option<bool>,
  #[new(default)]
  pub requires_signature: Option<bool>,

  // Constraints
  #[new(default)]
  pub vehicle_required: Option<app_108jobs_db_schema_file::enums::VehicleType>,
  #[new(default)]
  pub latest_pickup_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub latest_dropoff_at: Option<DateTime<Utc>>,

  // Contacts
  #[new(default)]
  pub sender_name: Option<String>,
  #[new(default)]
  pub sender_phone: Option<String>,
  #[new(default)]
  pub receiver_name: Option<String>,
  #[new(default)]
  pub receiver_phone: Option<String>,

  // Payment options
  #[new(default)]
  pub cash_on_delivery: Option<bool>,
  #[new(default)]
  pub cod_amount: Option<f64>,

  // Tracking state
  #[new(default)]
  pub status: Option<DeliveryStatus>,
}

#[cfg(feature = "full")]
impl DeliveryDetailsInsertForm {
  pub async fn insert(&self, pool: &mut utils::DbPool<'_>) -> Result<(), diesel::result::Error> {
    use app_108jobs_db_schema_file::schema::delivery_details::dsl;
    // Explicitly unwrap connection to avoid type inference issues
    let mut conn = match utils::get_conn(pool).await {
      Ok(c) => c,
      Err(e) => return Err(e),
    };

    diesel::insert_into(dsl::delivery_details)
      .values(self)
      .on_conflict(dsl::post_id)
      .do_update()
      .set(self)
      .execute(&mut conn)
      .await
      .map(|_| ())
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = delivery_details))]
#[serde(rename_all = "camelCase")]
pub struct DeliveryDetailsUpdateForm {
  // Locations
  pub pickup_address: Option<String>,
  pub pickup_lat: Option<Option<f64>>,
  pub pickup_lng: Option<Option<f64>>,
  pub dropoff_address: Option<String>,
  pub dropoff_lat: Option<Option<f64>>,
  pub dropoff_lng: Option<Option<f64>>,

  // Package
  pub package_description: Option<Option<String>>,
  pub package_weight_kg: Option<Option<f64>>,
  pub package_size: Option<Option<String>>,
  pub fragile: Option<bool>,
  pub requires_signature: Option<bool>,

  // Constraints
  pub vehicle_required: Option<Option<app_108jobs_db_schema_file::enums::VehicleType>>,
  pub latest_pickup_at: Option<Option<DateTime<Utc>>>,
  pub latest_dropoff_at: Option<Option<DateTime<Utc>>>,

  // Contacts
  pub sender_name: Option<Option<String>>,
  pub sender_phone: Option<Option<String>>,
  pub receiver_name: Option<Option<String>>,
  pub receiver_phone: Option<Option<String>>,

  // Payment
  pub cash_on_delivery: Option<bool>,
  pub cod_amount: Option<Option<f64>>,

  // Status
  pub status: Option<DeliveryStatus>,

  // Metadata
  pub updated_at: Option<DateTime<Utc>>,
}
