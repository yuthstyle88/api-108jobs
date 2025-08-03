use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Get wallet for a user.
pub struct GetWallet {
  pub user_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Update wallet balance.
pub struct UpdateWallet {
  pub amount: f64,
}