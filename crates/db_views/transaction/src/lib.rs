#[cfg(feature = "full")]
pub mod impls;

use lemmy_db_schema::source::transaction::Transaction;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts-rs")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub struct TransactionView {
    pub transaction: Transaction,
}