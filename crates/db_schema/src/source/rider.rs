use crate::newtypes::{LocalUserId, PersonId, RiderId};
use app_108jobs_db_schema_file::enums::{RiderVerificationStatus, VehicleType};
#[cfg(feature = "full")]
use app_108jobs_db_schema_file::schema::rider;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = rider))]
#[cfg_attr(feature = "full", diesel(primary_key(id)))]
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
