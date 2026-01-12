use crate::newtypes::{LocalUserId, PersonId, RiderId};
use app_108jobs_db_schema_file::enums::{RiderVerificationStatus, VehicleType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {app_108jobs_db_schema_file::schema::rider, i_love_jesus::CursorKeysModule};

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = rider))]
#[cfg_attr(feature = "full", diesel(primary_key(id)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = rider_keys))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct Rider {
  pub id: RiderId,

  /// References
  pub user_id: LocalUserId,
  pub person_id: PersonId,

  /// Vehicle
  pub vehicle_type: VehicleType,
  pub vehicle_plate_number: Option<String>,
  pub license_number: Option<String>,
  pub license_expiry_date: Option<DateTime<Utc>>,

  /// Verification
  pub is_verified: bool,
  pub is_active: bool,
  pub verification_status: RiderVerificationStatus,

  /// Performance
  pub rating: f64,
  pub completed_jobs: i32,
  pub total_jobs: i32,
  pub total_earnings: f64,
  pub pending_earnings: f64,

  /// Availability
  pub is_online: bool,
  pub accepting_jobs: bool,

  /// Timestamps
  pub joined_at: Option<DateTime<Utc>>,
  pub last_active_at: Option<DateTime<Utc>>,
  pub verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(
  feature = "full",
  derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
#[cfg_attr(feature = "full", diesel(table_name = rider))]
pub struct RiderInsertForm {
  /// References
  pub user_id: LocalUserId,
  pub person_id: PersonId,

  /// Vehicle
  pub vehicle_type: VehicleType,
  #[new(default)]
  pub vehicle_plate_number: Option<String>,
  #[new(default)]
  pub license_number: Option<String>,
  #[new(default)]
  pub license_expiry_date: Option<DateTime<Utc>>,

  /// Verification
  #[new(default)]
  pub is_verified: Option<bool>,
  #[new(default)]
  pub is_active: Option<bool>,
  #[new(default)]
  pub verification_status: Option<RiderVerificationStatus>,

  /// Performance
  #[new(default)]
  pub rating: Option<f64>,
  #[new(default)]
  pub completed_jobs: Option<i32>,
  #[new(default)]
  pub total_jobs: Option<i32>,
  #[new(default)]
  pub total_earnings: Option<f64>,
  #[new(default)]
  pub pending_earnings: Option<f64>,

  /// Availability
  #[new(default)]
  pub is_online: Option<bool>,
  #[new(default)]
  pub accepting_jobs: Option<bool>,

  /// Timestamps
  #[new(default)]
  pub joined_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub last_active_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset, Serialize, Deserialize))]
#[cfg_attr(feature = "full", diesel(table_name = rider))]
pub struct RiderUpdateForm {
  /// Vehicle
  pub vehicle_type: Option<VehicleType>,
  pub vehicle_plate_number: Option<Option<String>>,
  pub license_number: Option<Option<String>>,
  pub license_expiry_date: Option<Option<DateTime<Utc>>>,

  /// Verification
  pub is_verified: Option<bool>,
  pub is_active: Option<bool>,
  pub verification_status: Option<RiderVerificationStatus>,

  /// Performance
  pub rating: Option<f64>,
  pub completed_jobs: Option<i32>,
  pub total_jobs: Option<i32>,
  pub total_earnings: Option<f64>,
  pub pending_earnings: Option<f64>,

  /// Availability
  pub is_online: Option<bool>,
  pub accepting_jobs: Option<bool>,

  /// Timestamps
  pub joined_at: Option<Option<DateTime<Utc>>>,
  pub last_active_at: Option<Option<DateTime<Utc>>>,
  pub verified_at: Option<Option<DateTime<Utc>>>,
}
