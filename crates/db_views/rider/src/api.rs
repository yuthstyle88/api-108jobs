use crate::RiderView;
use app_108jobs_db_schema::newtypes::PaginationCursor;
use app_108jobs_db_schema::newtypes::RiderId;
use app_108jobs_db_schema_file::enums::VehicleType;
use app_108jobs_utils::error::{FastJobError, FastJobResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRiderRequest {
  pub vehicle_type: VehicleType,
  pub vehicle_plate_number: Option<String>,
  pub license_number: Option<String>,
  pub license_expiry_date: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct CreateRider {
  pub vehicle_type: VehicleType,
  pub vehicle_plate_number: Option<String>,
  pub license_number: Option<String>,
  pub license_expiry_date: Option<DateTime<Utc>>,
}

impl TryFrom<CreateRiderRequest> for CreateRider {
  type Error = FastJobError;

  fn try_from(value: CreateRiderRequest) -> FastJobResult<Self> {
    Ok(Self {
      vehicle_type: value.vehicle_type,
      vehicle_plate_number: value.vehicle_plate_number,
      license_number: value.license_number,
      license_expiry_date: value.license_expiry_date,
    })
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GetRider {
  pub id: Option<RiderId>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GetRiderResponse {
  pub rider_view: RiderView,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRiders {
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
  pub online_only: Option<bool>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListRidersResponse {
  pub riders: Vec<RiderView>,
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}
