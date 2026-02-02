use crate::newtypes::{CommentId, DeliveryDetailsId, PersonId, PostId, RiderId};
use crate::utils;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  app_108jobs_db_schema_file::schema::delivery_details,
  diesel::prelude::*,
  diesel_async::RunQueryDsl,
};
use app_108jobs_db_schema_file::enums::{DeliveryStatus, VehicleType};

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
  pub cancellation_reason: Option<String>,
  pub assigned_rider_id: Option<RiderId>,
  pub assigned_at: Option<DateTime<Utc>>,
  pub assigned_by_person_id: Option<PersonId>,
  pub linked_comment_id: Option<CommentId>,
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
  pub vehicle_required: Option<VehicleType>,
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
  pub cancellation_reason: Option<Option<String>>,

  // Assignment
  pub assigned_rider_id: Option<Option<RiderId>>,
  pub assigned_at: Option<Option<DateTime<Utc>>>,
  pub assigned_by_person_id: Option<Option<PersonId>>,
  pub linked_comment_id: Option<Option<CommentId>>,

  // Metadata
  pub updated_at: Option<DateTime<Utc>>,
}

/// Payload for updating delivery details via API.
/// This is a flattened version without nested Option<> for easier API usage.
/// Only fields that should be updatable after post creation are included.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct DeliveryDetailsPayload {
  // Locations
  pub pickup_address: Option<String>,
  pub pickup_lat: Option<f64>,
  pub pickup_lng: Option<f64>,
  pub dropoff_address: Option<String>,
  pub dropoff_lat: Option<f64>,
  pub dropoff_lng: Option<f64>,

  // Package
  pub package_description: Option<String>,
  pub package_weight_kg: Option<f64>,
  pub package_size: Option<String>,
  pub fragile: Option<bool>,
  pub requires_signature: Option<bool>,

  // Constraints
  pub vehicle_required: Option<VehicleType>,
  pub latest_pickup_at: Option<DateTime<Utc>>,
  pub latest_dropoff_at: Option<DateTime<Utc>>,

  // Contacts
  pub sender_name: Option<String>,
  pub sender_phone: Option<String>,
  pub receiver_name: Option<String>,
  pub receiver_phone: Option<String>,

  // Payment
  pub cash_on_delivery: Option<bool>,
  pub cod_amount: Option<f64>,
}

#[cfg(feature = "full")]
impl DeliveryDetailsPayload {
  /// Convert the payload to DeliveryDetailsUpdateForm.
  /// This wraps fields in Option<> where needed for the update form.
  pub fn to_update_form(self) -> DeliveryDetailsUpdateForm {
    DeliveryDetailsUpdateForm {
      pickup_address: self.pickup_address,
      pickup_lat: self.pickup_lat.map(Some),
      pickup_lng: self.pickup_lng.map(Some),
      dropoff_address: self.dropoff_address,
      dropoff_lat: self.dropoff_lat.map(Some),
      dropoff_lng: self.dropoff_lng.map(Some),
      package_description: self.package_description.map(Some),
      package_weight_kg: self.package_weight_kg.map(Some),
      package_size: self.package_size.map(Some),
      fragile: self.fragile,
      requires_signature: self.requires_signature,
      vehicle_required: self.vehicle_required.map(Some),
      latest_pickup_at: self.latest_pickup_at.map(Some),
      latest_dropoff_at: self.latest_dropoff_at.map(Some),
      sender_name: self.sender_name.map(Some),
      sender_phone: self.sender_phone.map(Some),
      receiver_name: self.receiver_name.map(Some),
      receiver_phone: self.receiver_phone.map(Some),
      cash_on_delivery: self.cash_on_delivery,
      cod_amount: self.cod_amount.map(Some),
      // The following fields are not editable via post update:
      status: None,
      cancellation_reason: None,
      assigned_rider_id: None,
      assigned_at: None,
      assigned_by_person_id: None,
      linked_comment_id: None,
      updated_at: Some(Utc::now()),
    }
  }
}
