use crate::newtypes::{LocalUserId, UserKeyBackupId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::user_key_backups;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = user_key_backups))]
#[cfg_attr(feature = "full", diesel(primary_key(id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[serde(rename_all = "camelCase")]
pub struct UserKeyBackup {
    pub id: UserKeyBackupId,
    pub local_user_id: LocalUserId,
    pub encrypted_private_key: Vec<u8>,
    pub iv: Vec<u8>,
    pub salt: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = user_key_backups))]
pub struct UserKeyBackupInsertForm {
    pub local_user_id: LocalUserId,
    pub encrypted_private_key: Vec<u8>,
    pub iv: Vec<u8>,
    pub salt: Vec<u8>,
    #[new(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[new(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = user_key_backups))]
pub struct UserKeyBackupUpdateForm {
    pub encrypted_private_key: Option<Vec<u8>>,
    pub iv: Option<Vec<u8>>,
    pub salt: Option<Vec<u8>>,
    pub updated_at: Option<DateTime<Utc>>,
}
