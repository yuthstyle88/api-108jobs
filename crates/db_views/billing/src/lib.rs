use lemmy_db_schema::source::billing::Billing;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BillingView {
  pub billings: Vec<Billing>,
}

pub mod impls;
pub mod api;