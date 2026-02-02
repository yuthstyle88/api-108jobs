use crate::RiderView;
use app_108jobs_db_schema::newtypes::{CommentId, PaginationCursor, PersonId, PostId};
use app_108jobs_db_schema::newtypes::RiderId;
use app_108jobs_db_schema_file::enums::{DeliveryStatus, VehicleType};
use app_108jobs_utils::error::{FastJobError, FastJobResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

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
pub struct ListRidersQuery {
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
  pub verified: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListRidersResponse {
  pub riders: Vec<RiderView>,
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminVerifyRiderRequest {
  pub rider_id: RiderId,
  /// approve = true will mark rider as verified; false will reject
  pub approve: bool,
  /// Optional reason when rejecting
  pub reason: Option<String>,
}

/// Request body for updating delivery status
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDeliveryStatusRequest {
  /// The new status to set
  pub status: DeliveryStatus,
  /// Optional reason for status change (required for cancellation)
  pub reason: Option<String>,
}

/// Response after updating delivery status
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryStatusResponse {
  pub post_id: PostId,
  pub status: DeliveryStatus,
  pub cancellation_reason: Option<String>,
  pub updated_at: DateTime<Utc>,
}

/// Event published to Redis for WebSocket clients
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryStatusEvent {
  #[serde(rename = "type")]
  pub kind: &'static str,
  pub post_id: PostId,
  pub status: DeliveryStatus,
  pub updated_at: DateTime<Utc>,
  pub reason: Option<String>,
}

/// Request body for assigning a delivery to a rider from a comment/proposal
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssignDeliveryRequest {
  /// The ID of the comment (proposal) that led to this assignment
  pub comment_id: CommentId,
  /// The ID of the rider's person (author of the comment) to assign
  pub person_id: PersonId,
  /// Sender's name (person who is sending the package)
  pub sender_name: String,
  /// Sender's phone number
  pub sender_phone: String,
  /// Receiver's name (person who will receive the package)
  pub receiver_name: String,
  /// Receiver's phone number
  pub receiver_phone: String,
}

/// Response after assigning a delivery to a rider
#[skip_serializing_none]
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssignDeliveryResponse {
  /// The post ID of the delivery
  pub post_id: PostId,
  /// The rider who was assigned
  pub rider_id: RiderId,
  /// When the assignment was made
  pub assigned_at: DateTime<Utc>,
  /// The person who made the assignment
  pub assigned_by_person_id: Option<i32>,
  /// The new delivery status
  pub status: DeliveryStatus,
  /// The linked comment (proposal) ID
  pub linked_comment_id: CommentId,
}

/// Event published to Redis when a delivery is assigned
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryAssignmentEvent {
  #[serde(rename = "type")]
  pub kind: &'static str,
  pub post_id: PostId,
  pub rider_id: RiderId,
  pub assigned_at: DateTime<Utc>,
  pub status: DeliveryStatus,
}

/// Response for getting a rider's current active delivery assignment
#[skip_serializing_none]
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MyDeliveryAssignmentResponse {
  /// The delivery details if rider has an active assignment
  pub delivery: Option<MyDeliveryInfo>,
}

/// Simplified delivery info for rider's current assignment
#[skip_serializing_none]
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MyDeliveryInfo {
  pub post_id: PostId,
  pub status: DeliveryStatus,
  pub pickup_address: String,
  pub dropoff_address: String,
  pub assigned_at: Option<DateTime<Utc>>,
  pub linked_comment_id: Option<CommentId>,
}