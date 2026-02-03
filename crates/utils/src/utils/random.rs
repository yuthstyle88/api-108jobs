/// Random number generation utilities.

/// Generate a random 5-digit number as a string (00000-99999).
pub fn rand_number5() -> Option<String> {
  Some(format!("{:05}", fastrand::u32(..100_000)))
}
