use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};
use std::str::FromStr;
use std::{
  fmt,
  fmt::{Display, Formatter},
  ops::Deref,
};
use url::Url;
#[cfg(feature = "full")]
use {
  diesel::{
    backend::Backend,
    deserialize::FromSql,
    pg::Pg,
    serialize::{Output, ToSql},
    sql_types::Text,
  },
  diesel_ltree::Ltree,
  app_108jobs_utils::error::{FastJobErrorType, FastJobResult},
};
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The post id.
pub struct PostId(pub i32);

impl fmt::Display for PostId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The chat message id.
pub struct ChatMessageId(pub i64);

#[derive(Debug, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The chat message id.
pub struct ChatMessageRefId(pub String);
impl fmt::Display for ChatMessageRefId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The chat room serial id.
pub struct SerialId(pub i64);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The person id.
pub struct PersonId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The comment id.
pub struct CommentId(pub i32);

impl fmt::Display for CommentId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

pub enum PostOrCommentId {
  Post(PostId),
  Comment(CommentId),
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The category id.
pub struct CategoryId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The local user id.
pub struct LocalUserId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The local pair user id.
pub struct LocalPairUserId(pub i32, pub i32);

impl TryFrom<String> for LocalPairUserId {
  type Error = &'static str;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    LocalPairUserId::try_from(value.as_str())
  }
}

impl TryFrom<&str> for LocalPairUserId {
  type Error = &'static str;
  fn try_from(topic: &str) -> Result<Self, Self::Error> {
    // Accept both "room:<roomId>:<senderId>:<receiverId>" and "<roomId>:<senderId>:<receiverId>"
    let cleaned = topic.strip_prefix("room:").unwrap_or(topic);
    let parts: Vec<&str> = cleaned.split(':').collect();
    if parts.len() != 3 {
      return Err("invalid channel format");
    }
    let sender_id = parts[1].parse::<i32>().map_err(|_| "invalid sender id")?;
    let receiver_id = parts[2].parse::<i32>().map_err(|_| "invalid receiver id")?;
    Ok(LocalPairUserId(sender_id, receiver_id))
  }
}
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The wallet id.
pub struct WalletId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The Top-up Request id.
pub struct TopUpRequestId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The Withdrawal Request id.
pub struct WithdrawRequestId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The billing id.
pub struct BillingId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The bank id.
pub struct BankId(pub i32);
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The Coin id.
pub struct CoinId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The Workflow id.
pub struct WorkflowId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The Job Budget Plan id.
pub struct JobBudgetPlanId(pub i32);
#[derive(
  Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The coin.
pub struct Coin(pub i32);

impl PartialEq<i32> for Coin {
  #[inline]
  fn eq(&self, other: &i32) -> bool {
    self.0 == *other
  }
}

impl PartialOrd<i32> for Coin {
  #[inline]
  fn partial_cmp(&self, other: &i32) -> Option<Ordering> {
    self.0.partial_cmp(other)
  }
}
impl Add for Coin {
  type Output = Coin;
  #[inline]
  fn add(self, rhs: Coin) -> Coin {
    Coin(self.0 + rhs.0)
  }
}

impl Sub for Coin {
  type Output = Coin;
  #[inline]
  fn sub(self, rhs: Coin) -> Coin {
    Coin(self.0 - rhs.0)
  }
}

impl AddAssign for Coin {
  #[inline]
  fn add_assign(&mut self, rhs: Coin) {
    self.0 += rhs.0;
  }
}

impl SubAssign for Coin {
  #[inline]
  fn sub_assign(&mut self, rhs: Coin) {
    self.0 -= rhs.0;
  }
}

impl Neg for Coin {
  type Output = Coin;
  #[inline]
  fn neg(self) -> Coin {
    Coin(-self.0)
  }
}

impl Neg for &Coin {
  type Output = Coin;
  #[inline]
  fn neg(self) -> Coin {
    Coin(-self.0)
  }
}
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The user bank account id.
pub struct BankAccountId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The identity card id.
pub struct IdentityCardId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The address id.
pub struct AddressId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The user review id.
pub struct UserReviewId(pub i32);

#[derive(Debug, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The chat room id.
pub struct ChatRoomId(pub String);
impl Display for ChatRoomId {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl ChatRoomId {
  /// Remove control and zero-width characters and trim whitespace.
  fn clean(s: &str) -> String {
    s.chars()
      .filter(|&c| {
        // Remove control chars (U+0000..U+001F, U+007F), and common zero-width (U+200B..U+200F, U+FEFF)
        !(c.is_control()
          || matches!(
            c,
            '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{200E}' | '\u{200F}' | '\u{FEFF}'
          ))
      })
      .collect::<String>()
      .trim()
      .to_string()
  }

  /// Extract a `ChatRoomId` from a Phoenix channel name (`room:...`).
  /// Supports multiple formats and validates the room id.
  pub fn from_channel_name(channel: &str) -> Result<Self, &'static str> {
    // Accept formats:
    // 1) "room:<roomId>:<senderId>:<receiverId>"
    // 2) "<roomId>:<senderId>:<receiverId>"
    // 3) legacy: "room:<roomId>" or "<roomId>"
    // 4) legacy: "<roomId>:<receiverId>"
    let cleaned = channel.strip_prefix("room:").unwrap_or(channel);
    let parts: Vec<&str> = cleaned.split(':').collect();

    let room_id = match parts.len() {
      3 => parts[0], // <roomId>:<senderId>:<receiverId>
      2 => parts[0], // legacy <roomId>:<receiverId>
      1 => parts[0], // legacy <roomId>
      _ => return Err("invalid channel format"),
    };

    if Self::is_valid_id(room_id) {
      Ok(ChatRoomId(room_id.to_string()))
    } else {
      Err("invalid room id format")
    }
  }

  fn is_valid_id(s: &str) -> bool {
    // Accept hex string of length 16 or UUID
    let is_hex = s.len() == 16 && s.chars().all(|c| matches!(c, 'a'..='f' | '0'..='9'));
    let is_uuid = {
      let parts: Vec<&str> = s.split('-').collect();
      parts.len() == 5
        && parts[0].len() == 8
        && parts[1].len() == 4
        && parts[2].len() == 4
        && parts[3].len() == 4
        && parts[4].len() == 12
        && s.chars().all(|c| c == '-' || c.is_ascii_hexdigit())
    };
    is_hex || is_uuid
  }
}

impl FromStr for ChatRoomId {
  type Err = &'static str;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    ChatRoomId::try_from(s)
  }
}

impl TryFrom<String> for ChatRoomId {
  type Error = &'static str;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    let cleaned = ChatRoomId::clean(&value);
    if ChatRoomId::is_valid_id(&cleaned) {
      Ok(ChatRoomId(cleaned))
    } else {
      Err("invalid room id format")
    }
  }
}

impl TryFrom<&str> for ChatRoomId {
  type Error = &'static str;
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let cleaned = ChatRoomId::clean(value);
    if ChatRoomId::is_valid_id(&cleaned) {
      Ok(ChatRoomId(cleaned))
    } else {
      Err("invalid room id format")
    }
  }
}

impl Deref for ChatRoomId {
  type Target = str;

  fn deref(&self) -> &str {
    &self.0
  }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct SharedSecret(pub String);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The person comment mention id.
pub struct PersonCommentMentionId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The person post mention id.
pub struct PersonPostMentionId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The comment report id.
pub struct CommentReportId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The category report id.
pub struct CategoryReportId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The post report id.
pub struct PostReportId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The site id.
pub struct SiteId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The language id.
pub struct LanguageId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The comment reply id.
pub struct CommentReplyId(pub i32);

#[derive(
  Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default, Ord, PartialOrd,
)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The instance id.
pub struct InstanceId(pub i32);

#[derive(
  Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default, PartialOrd, Ord,
)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ActivityId(pub i64);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The local site id.
pub struct LocalSiteId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The custom emoji id.
pub struct CustomEmojiId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The tagline id.
pub struct TaglineId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The registration application id.
pub struct RegistrationApplicationId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The oauth provider id.
pub struct OAuthProviderId(pub i32);

#[cfg(feature = "full")]
#[derive(Serialize, Deserialize)]
#[serde(remote = "Ltree")]
/// Do remote derivation for the Ltree struct
pub struct LtreeDef(pub String);

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
#[cfg_attr(feature = "full", derive(AsExpression, FromSqlRow))]
#[cfg_attr(feature = "full", diesel(sql_type = diesel::sql_types::Text))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct DbUrl(pub(crate) Box<Url>);

impl TryFrom<&str> for DbUrl {
  type Error = url::ParseError;

  fn try_from(s: &str) -> Result<Self, Self::Error> {
    Ok(DbUrl(Box::new(Url::parse(s)?)))
  }
}

impl FromStr for DbUrl {
  type Err = url::ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Url::parse(s).map(|url| DbUrl(Box::new(url)))
  }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
/// The report combined id
pub struct ReportCombinedId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
/// The person content combined id
pub struct PersonContentCombinedId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
/// The person saved combined id
pub struct PersonSavedCombinedId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
/// The person liked combined id
pub struct PersonLikedCombinedId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct ModlogCombinedId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
/// The inbox combined id
pub struct InboxCombinedId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
/// The search combined id
pub struct SearchCombinedId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminAllowInstanceId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminBlockInstanceId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminPurgePersonId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminPurgeCategoryId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminPurgeCommentId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminPurgePostId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ModRemovePostId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ModRemoveCommentId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ModRemoveCategoryId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ModLockPostId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ModFeaturePostId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ModBanFromCategoryId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ModBanId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ModChangeCategoryVisibilityId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ModAddCategoryId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ModTransferCategoryId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ModAddId(pub i32);

impl DbUrl {
  pub fn inner(&self) -> &Url {
    &self.0
  }
}

impl Display for DbUrl {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.clone().0.fmt(f)
  }
}

// the project doesn't compile with From
#[expect(clippy::from_over_into)]
impl Into<DbUrl> for Url {
  fn into(self) -> DbUrl {
    DbUrl(Box::new(self))
  }
}
#[expect(clippy::from_over_into)]
impl Into<Url> for DbUrl {
  fn into(self) -> Url {
    *self.0
  }
}

impl Deref for DbUrl {
  type Target = Url;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[cfg(feature = "full")]
impl ToSql<Text, Pg> for DbUrl {
  fn to_sql(&self, out: &mut Output<Pg>) -> diesel::serialize::Result {
    <std::string::String as ToSql<Text, Pg>>::to_sql(&self.0.to_string(), &mut out.reborrow())
  }
}

#[cfg(feature = "full")]
impl<DB: Backend> FromSql<Text, DB> for DbUrl
where
  String: FromSql<Text, DB>,
{
  fn from_sql(value: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
    let str = String::from_sql(value)?;
    Ok(DbUrl(Box::new(Url::parse(&str)?)))
  }
}

impl InstanceId {
  pub fn inner(self) -> i32 {
    self.0
  }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The internal tag id.
pub struct TagId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The Skill id.
pub struct SkillId(pub i32);

/// A pagination cursor
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct PaginationCursor(pub String);

pub enum DecodedCursor {
  I32(i32),
  I64(i64),
  Composite(Vec<(char, i32)>),
}

#[cfg(feature = "full")]
#[cfg(feature = "full")]
impl PaginationCursor {
  pub fn new_single(prefix: char, id: i32) -> Self {
    Self::new_i64(prefix, id as i64)
  }

  pub fn new(prefixes_and_ids: &[(char, i32)]) -> Self {
    Self(
      prefixes_and_ids
        .iter()
        .map(|(prefix, id)| format!("{prefix}{id:x}"))
        .collect::<Vec<String>>()
        .join("-"),
    )
  }

  pub fn new_i64(prefix: char, id: i64) -> Self {
    let high = (id >> 32) as i32;
    let low = id as i32;

    // Build exactly the same way the old `new()` does
    let parts = [
      format!("{prefix}{high:x}"),
      format!("{}{low:x}", Self::next_prefix(prefix)),
    ];
    Self(parts.join("-"))
  }

  // Helper – deterministic second prefix so we know it’s an i64 cursor
  fn next_prefix(p: char) -> char {
    // 'M' → 'N', 'R' → 'S', etc. Safe because we only use ASCII letters
    char::from_u32(p as u32 + 1).unwrap_or('Z')
  }

  // ────── Decoding ──────
  pub fn prefixes_and_ids(&self) -> Vec<(char, i32)> {
    let default_prefix = 'Z';
    let default_id = 0;
    self
      .0
      .split('-')
      .map(|segment| {
        if segment.is_empty() {
          return (default_prefix, default_id);
        }
        let (prefix_str, id_str) = segment.split_at(1);
        let prefix = prefix_str.chars().next().unwrap_or(default_prefix);
        let id = i32::from_str_radix(id_str, 16).unwrap_or(default_id);
        (prefix, id)
      })
      .collect()
  }

  pub fn first_id(&self) -> FastJobResult<i32> {
    self
      .prefixes_and_ids()
      .first()
      .map(|&(_, id)| id)
      .ok_or(FastJobErrorType::CouldntParsePaginationToken.into())
  }

  // ────── NEW: safe i64 decoder (backward compatible) ──────
  pub fn as_i64(&self) -> FastJobResult<i64> {
    let parts = self.prefixes_and_ids();
    match parts.len() {
      1 => Ok(parts[0].1 as i64), // old i32 cursor
      2 => {
        let high = parts[0].1 as i64;
        let low = parts[1].1 as i64;
        Ok((high << 32) | low)
      }
      _ => Err(FastJobErrorType::CouldntParsePaginationToken.into()),
    }
  }

  /* ───────── i32 ───────── */
  pub fn v2_i32(id: i32) -> Self {
    Self(format!("v2:I:{:x}", id))
  }

  /* ───────── i64 ───────── */
  pub fn v2_i64(id: i64) -> Self {
    let high = (id >> 32) as u32;
    let low = id as u32;
    Self(format!("v2:L:{:x}:{:x}", high, low))
  }

  /* ───────── composite ───────── */
  pub fn v2_composite(parts: &[(char, i32)]) -> Self {
    let encoded = parts
        .iter()
        .map(|(p, id)| format!("{p}{:x}", id))
        .collect::<Vec<_>>()
        .join(",");
    Self(format!("v2:C:{encoded}"))
  }

  pub fn decode(&self) -> FastJobResult<DecodedCursor> {
    if !self.0.starts_with("v2:") {
      return self.decode_legacy();
    }

    let payload = &self.0[3..];
    let mut it = payload.split(':');

    match it.next() {
      Some("I") => {
        let id = i32::from_str_radix(
          it.next().ok_or(FastJobErrorType::CouldntParsePaginationToken)?,
          16,
        )?;
        Ok(DecodedCursor::I32(id))
      }

      Some("L") => {
        let high = u32::from_str_radix(it.next().ok_or(FastJobErrorType::CouldntParsePaginationToken)?, 16)? as i64;
        let low = u32::from_str_radix(it.next().ok_or(FastJobErrorType::CouldntParsePaginationToken)?, 16)? as i64;
        Ok(DecodedCursor::I64((high << 32) | low))
      }

      Some("C") => {
        let parts = it
            .next()
            .ok_or(FastJobErrorType::CouldntParsePaginationToken)?
            .split(',')
            .map(|s| {
              let (p, rest) = s.split_at(1);
              let id = i32::from_str_radix(rest, 16)?;
              Ok((p.chars().next().unwrap(), id))
            })
            .collect::<FastJobResult<Vec<_>>>()?;

        Ok(DecodedCursor::Composite(parts))
      }

      _ => Err(FastJobErrorType::CouldntParsePaginationToken.into()),
    }
  }

  fn decode_legacy(&self) -> FastJobResult<DecodedCursor> {
    let parts = self
        .0
        .split('-')
        .map(|segment| {
          let (p, id) = segment.split_at(1);
          let id = i32::from_str_radix(id, 16)?;
          Ok((p.chars().next().unwrap(), id))
        })
        .collect::<FastJobResult<Vec<_>>>()?;

    if parts.len() == 1 {
      Ok(DecodedCursor::I32(parts[0].1))
    } else if parts.len() == 2 {
      let high = parts[0].1 as i64;
      let low = parts[1].1 as i64;
      Ok(DecodedCursor::I64((high << 32) | low))
    } else {
      Ok(DecodedCursor::Composite(parts))
    }
  }
}
