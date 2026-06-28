use crate::newtypes::{DeliveryRiderRatingId, PersonId, PostId, RiderId};
#[cfg(feature = "full")]
use crate::schema::delivery_rider_rating;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Employer rating of a rider for a completed delivery.
#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = delivery_rider_rating))]
#[cfg_attr(feature = "full", diesel(primary_key(id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[serde(rename_all = "camelCase")]
pub struct DeliveryRiderRating {
  pub id: DeliveryRiderRatingId,
  pub post_id: PostId,
  pub employer_id: PersonId,
  pub rider_id: RiderId,
  pub rating: i16,
  pub proposal: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = delivery_rider_rating))]
pub struct DeliveryRiderRatingInsertForm {
  pub post_id: PostId,
  pub employer_id: PersonId,
  pub rider_id: RiderId,
  pub rating: i16,
  #[new(default)]
  pub proposal: Option<String>,
  #[new(default)]
  pub created_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub updated_at: Option<Option<DateTime<Utc>>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = delivery_rider_rating))]
pub struct DeliveryRiderRatingUpdateForm {
  pub rating: Option<i16>,
  pub proposal: Option<Option<String>>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
}
