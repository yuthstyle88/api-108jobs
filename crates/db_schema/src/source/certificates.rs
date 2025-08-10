use crate::newtypes::{CertificateId, DbUrl, PersonId};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::certificates;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = certificates))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct Certificates {
    pub id: CertificateId,
    pub person_id: PersonId,
    pub name: String,
    pub achieved_date: NaiveDate,
    pub expires_date: NaiveDate,
    pub url: Option<DbUrl>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = certificates))]
pub struct CertificatesInsertForm {
    pub person_id: PersonId,
    pub name: String,
    pub achieved_date: Option<NaiveDate>,
    pub expires_date: Option<NaiveDate>,
    pub url: Option<DbUrl>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = certificates))]
pub struct CertificatesUpdateForm {
    pub name: Option<String>,
    pub achieved_date: Option<NaiveDate>,
    pub expires_date: Option<NaiveDate>,
    pub url: Option<Option<DbUrl>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct CertificateItem {
    pub id: Option<CertificateId>, // None for new items, Some(id) for updates
    pub name: String,
    pub achieved_date: Option<NaiveDate>, // Date as string in YYYY-MM-DD format
    pub expires_date: Option<NaiveDate>,  // Date as string in YYYY-MM-DD format
    pub url: Option<DbUrl>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = certificates))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct CertificateView {
    pub id: CertificateId, // None for new items, Some(id) for updates
    pub name: String,
    pub achieved_date: NaiveDate, // Date as string in YYYY-MM-DD format
    pub expires_date: NaiveDate,  // Date as string in YYYY-MM-DD format
    pub url: DbUrl,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct CertificatesRequest {
    pub certificates: Vec<CertificateItem>,
}
