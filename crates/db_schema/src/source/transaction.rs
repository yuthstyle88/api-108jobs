use crate::newtypes::{BillingId, PersonId, PostId, TransactionId};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use diesel::{Insertable, Queryable, Selectable};
use lemmy_db_schema_file::enums::{TransactionStatus, TransactionType};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use serde_json::Value;
#[cfg(feature = "ts-rs")]
use ts_rs::TS;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = lemmy_db_schema_file::schema::transaction))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub struct Transaction {
    pub id: TransactionId,
    pub from_user_id: Option<PersonId>,
    pub to_user_id: Option<PersonId>,
    pub amount: BigDecimal,
    pub transaction_type: TransactionType,
    pub status: TransactionStatus,
    pub billing_id: Option<BillingId>,
    pub post_id: Option<PostId>,
    pub description: String,
    pub reference_number: Option<String>,
    #[cfg(feature = "full")]
    pub metadata: Option<Value>,
    #[cfg(not(feature = "full"))]
    pub metadata: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = lemmy_db_schema_file::schema::transaction))]
pub struct TransactionInsertForm {
    pub from_user_id: Option<PersonId>,
    pub to_user_id: Option<PersonId>,
    pub amount: BigDecimal,
    pub transaction_type: TransactionType,
    pub status: Option<TransactionStatus>,
    pub billing_id: Option<BillingId>,
    pub post_id: Option<PostId>,
    pub description: String,
    pub reference_number: Option<String>,
    #[cfg(feature = "full")]
    pub metadata: Option<Value>,
    #[cfg(not(feature = "full"))]
    pub metadata: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(diesel::AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = lemmy_db_schema_file::schema::transaction))]
pub struct TransactionUpdateForm {
    pub status: Option<TransactionStatus>,
    pub completed_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    #[cfg(feature = "full")]
    pub metadata: Option<Option<Value>>,
    #[cfg(not(feature = "full"))]
    pub metadata: Option<Option<String>>,
    pub reference_number: Option<Option<String>>,
}