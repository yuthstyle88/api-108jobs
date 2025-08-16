use crate::newtypes::{JobBudgetPlanId, PostId, Coin};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::job_budget_plan;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = job_budget_plan))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct JobBudgetPlan {
  pub id: JobBudgetPlanId,
  pub post_id: PostId,
  pub total_amount: Coin,
  #[cfg_attr(feature = "ts-rs", ts(type = "any"))]
  pub installments: JsonValue,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = job_budget_plan))]
pub struct JobBudgetPlanInsertForm {
  pub post_id: PostId,
  #[new(default)]
  pub total_amount: Option<Coin>,
  #[new(default)]
  pub installments: Option<JsonValue>,
  #[new(default)]
  pub created_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub updated_at: Option<Option<DateTime<Utc>>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = job_budget_plan))]
pub struct JobBudgetPlanUpdateForm {
  pub total_amount: Option<Coin>,
  pub installments: Option<JsonValue>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
}
