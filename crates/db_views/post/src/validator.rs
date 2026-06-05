//! Validation logic for post requests
use crate::api::{CreatePost, CreatePostRequest, EditPost, EditPostRequest};
use app_108jobs_db_schema_file::enums::PostKind;
use app_108jobs_utils::error::{FastJobError, FastJobErrorType, FastJobResult};
use chrono::Utc;
use url::Url;

use app_108jobs_utils::settings::SETTINGS;
use slug::slugify;

fn is_valid_post_title(title: &str) -> FastJobResult<()> {
  if title.trim().is_empty() {
    return Err(FastJobErrorType::InvalidField(
      "title cannot be empty".to_string(),
    ))?;
  }
  Ok(())
}

fn validate_job_update_fields(data: &CreatePostRequest) -> FastJobResult<()> {
  // Validate budget (now required)
  if data.budget.0 <= 0 {
    return Err(FastJobErrorType::InvalidField(
      "budget must be greater than 0".to_string(),
    ))?;
  }

  // Validate deadline: must be in the future, if set
  if let Some(deadline) = data.deadline {
    if deadline <= Utc::now() {
      return Err(FastJobErrorType::InvalidField(
        "deadline must be in the future".to_string(),
      ))?;
    }
  }

  // For Normal posts, category_id is required; for Delivery/RideTaxi it can be None
  if matches!(data.post_kind.unwrap_or(PostKind::Normal), PostKind::Normal)
    && data.category_id.is_none()
  {
    return Err(FastJobErrorType::InvalidDeliveryPost)?;
  }

  // Delivery-specific validation (request level only)
  if let Some(kind) = data.post_kind {
    if matches!(kind, PostKind::Delivery) {
      let dd = data
        .delivery_details
        .as_ref()
        .ok_or(FastJobErrorType::InvalidDeliveryPost)?;
      if dd
        .pickup_address
        .as_ref()
        .map_or(true, |s| s.trim().is_empty())
        || dd
          .dropoff_address
          .as_ref()
          .map_or(true, |s| s.trim().is_empty())
      {
        return Err(FastJobErrorType::InvalidDeliveryPost)?;
      }
      if let Some(true) = dd.cash_on_delivery {
        if dd.cod_amount.unwrap_or(0.0) <= 0.0 {
          return Err(FastJobErrorType::InvalidField(
            "cod_amount must be > 0 when cash_on_delivery is true".to_string(),
          ))?;
        }
      }
      // Basic sanity on lat/lng if present
      let ok = |lat: f64, lng: f64| {
        lat.is_finite() && lng.is_finite() && lat.abs() <= 90.0 && lng.abs() <= 180.0
      };
      if let (Some(lat), Some(lng)) = (dd.pickup_lat, dd.pickup_lng) {
        if !ok(lat, lng) {
          return Err(FastJobErrorType::InvalidField(
            "invalid pickup lat/lng".to_string(),
          ))?;
        }
      }
      if let (Some(lat), Some(lng)) = (dd.dropoff_lat, dd.dropoff_lng) {
        if !ok(lat, lng) {
          return Err(FastJobErrorType::InvalidField(
            "invalid dropoff lat/lng".to_string(),
          ))?;
        }
      }
    } else if matches!(kind, PostKind::RideTaxi) {
      let r = data
        .ride_payload
        .as_ref()
        .ok_or(FastJobErrorType::InvalidField(
          "ride_payload required for RideTaxi post".to_string(),
        ))?;

      if r.pickup_address.trim().is_empty() || r.dropoff_address.trim().is_empty() {
        return Err(FastJobErrorType::InvalidField(
          "pickup_address and dropoff_address are required".to_string(),
        ))?;
      }
      let ok = |lat: f64, lng: f64| {
        lat.is_finite() && lng.is_finite() && lat.abs() <= 90.0 && lng.abs() <= 180.0
      };
      if let (Some(lat), Some(lng)) = (r.pickup_lat, r.pickup_lng) {
        if !ok(lat, lng) {
          return Err(FastJobErrorType::InvalidField(
            "invalid pickup lat/lng".to_string(),
          ))?;
        }
      }
      if let (Some(lat), Some(lng)) = (r.dropoff_lat, r.dropoff_lng) {
        if !ok(lat, lng) {
          return Err(FastJobErrorType::InvalidField(
            "invalid dropoff lat/lng".to_string(),
          ))?;
        }
      }
      if data.delivery_details.is_some() {
        return Err(FastJobErrorType::InvalidField(
          "delivery_details not allowed for RideTaxi post".to_string(),
        ))?;
      }
    } else {
      if data.delivery_details.is_some() || data.ride_payload.is_some() {
        return Err(FastJobErrorType::InvalidField(
          "logistics payload not allowed for Normal post".to_string(),
        ))?;
      }
    }
  }
  Ok(())
}

impl TryFrom<CreatePostRequest> for CreatePost {
  type Error = FastJobError;
  fn try_from(data: CreatePostRequest) -> Result<Self, Self::Error> {
    is_valid_post_title(&data.name)?;
    validate_job_update_fields(&data)?;
    if let Some(ref url_str) = data.url {
      Url::parse(url_str).map_err(|_| FastJobErrorType::InvalidUrl)?;
    }

    if let Some(ref thumb_url) = data.custom_thumbnail {
      Url::parse(thumb_url).map_err(|_| FastJobErrorType::InvalidUrl)?;
    }

    let domain = SETTINGS.get_protocol_and_hostname();
    let raw_url = format!("{}/post/{}", domain, slugify(data.name.clone()));
    let url = Url::parse(&raw_url)?;

    // Determine post kind, defaulting to Normal for backward compatibility
    let post_kind = data.post_kind.unwrap_or(PostKind::Normal);

    Ok(CreatePost {
      name: data.name,
      category_id: data.category_id,
      url: data.url,
      body: data.body,
      alt_text: data.alt_text,
      language_id: data.language_id,
      custom_thumbnail: data.custom_thumbnail,
      honeypot: None,
      self_promotion: None,
      tags: None,
      scheduled_publish_time_at: None,
      budget: data.budget,
      deadline: data.deadline,
      intended_use: data.intended_use,
      job_type: data.job_type,
      is_english_required: data.is_english_required,
      ap_id: Some(url.into()),
      post_kind,
      delivery_details: data.delivery_details,
      ride_payload: data.ride_payload,
    })
  }
}

impl TryFrom<EditPostRequest> for EditPost {
  type Error = FastJobError;
  fn try_from(data: EditPostRequest) -> Result<Self, Self::Error> {
    if let Some(ref url_str) = data.url {
      Url::parse(url_str).map_err(|_| FastJobErrorType::InvalidUrl)?;
    }

    if let Some(ref thumb_url) = data.custom_thumbnail {
      Url::parse(thumb_url).map_err(|_| FastJobErrorType::InvalidUrl)?;
    }

    Ok(EditPost {
      post_id: data.post_id,
      category_id: data.category_id,
      name: data.name,
      url: data.url,
      body: data.body,
      alt_text: data.alt_text,
      language_id: data.language_id,
      custom_thumbnail: data.custom_thumbnail,
      self_promotion: None,
      tags: None,
      scheduled_publish_time_at: None,
      budget: data.budget,
      deadline: data.deadline,
      intended_use: data.intended_use,
      job_type: data.job_type,
      is_english_required: data.is_english_required,
      delivery_details: data.delivery_details,
    })
  }
}
