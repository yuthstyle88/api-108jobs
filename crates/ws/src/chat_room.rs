use base62::{decode, encode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatRoomTemp {
  pub user_a: i32,
  pub user_b: i32,
  pub job_id: i32,
  pub room_id: String,
}

impl ChatRoomTemp {
  /// Creates a new chat room ensuring consistent ordering of user IDs
  pub fn new(user_a: i32, user_b: i32, job_id: i32) -> Self {
    // Ensure consistent ordering to prevent duplicate rooms
    let (first_user, second_user) = if user_a < user_b {
      (user_a, user_b)
    } else {
      (user_b, user_a)
    };

    let room_id = Self::generate_compact_room_id(first_user, second_user, job_id);

    ChatRoomTemp {
      user_a: first_user,
      user_b: second_user,
      job_id,
      room_id,
    }
  }

  /// Generates a compact Base62 encoded room ID
  fn generate_compact_room_id(user_a: i32, user_b: i32, job_id: i32) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&user_a.to_be_bytes());
    bytes.extend_from_slice(&user_b.to_be_bytes());
    bytes.extend_from_slice(&job_id.to_be_bytes());

    // Convert the 12 bytes to a u128
    // Pad with zeros to make it 16 bytes (u128 size)
    let mut padded_bytes = [0u8; 16];
    padded_bytes[4..].copy_from_slice(&bytes);

    let num = u128::from_be_bytes(padded_bytes);
    encode(num)
  }

  /// Parses a compact Base62 room ID back into its constituent parts
  pub fn parse_compact_room_id(room_id: &str) -> Option<(i32, i32, i32)> {
    let num = decode(room_id).ok()?;
    let bytes = num.to_be_bytes();

    // Extract the original 12 bytes (skip first 4 padding bytes)
    let user_a = i32::from_be_bytes(bytes[4..8].try_into().ok()?);
    let user_b = i32::from_be_bytes(bytes[8..12].try_into().ok()?);
    let job_id = i32::from_be_bytes(bytes[12..16].try_into().ok()?);

    Some((user_a.min(user_b), user_b.max(user_a), job_id))
  }
}
