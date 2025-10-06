use crate::newtypes::{LocalUserId, UserKeyMappingId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::user_key_mappings;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = user_key_mappings))]
#[cfg_attr(feature = "full", diesel(primary_key(id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[serde(rename_all = "camelCase")]
pub struct UserKeyMapping {
    pub id: UserKeyMappingId,
    pub local_user_id: LocalUserId,
    pub hashed_password: String,
    pub real_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = user_key_mappings))]
pub struct UserKeyMappingInsertForm {
    pub local_user_id: LocalUserId,
    pub hashed_password: String,
    pub real_key: String,
    #[new(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[new(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = user_key_mappings))]
pub struct UserKeyMappingUpdateForm {
    pub hashed_password: Option<String>,
    pub real_key: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}
