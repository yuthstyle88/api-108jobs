use std::str::FromStr;
use unic_langid::LanguageIdentifier;

pub fn rand_number5() -> Option<String> {
   Some(format!("{:05}", fastrand::u32(..100_000)))
}

/// Convert a language code (ISO 639) to a default country code (ISO 3166).
/// Returns `None` if unknown or invalid.
pub fn lang_to_country_code(lang: Option<&str>) -> Option<String> {
   lang.and_then(|code| {
      if let Ok(mut langid) = LanguageIdentifier::from_str(code) {
         if langid.region.is_none() {
            match code {
               "vi" => langid.region = Some("VN".parse().unwrap()),
               "th" => langid.region = Some("TH".parse().unwrap()),
               _ => {}
            }
         }
         langid.region.map(|r| r.to_string())
      } else {
         None
      }
   })
}

#[inline]
pub fn contacts_key(user_id: i32) -> String {
   format!("contacts:user:{}", user_id)
}

#[inline]
pub fn rooms_key(user_id: i32) -> String {
   format!("rooms:user:{}", user_id)
}

#[inline]
pub fn presence_conn_key(user_id: i32, conn_id: &str) -> String {
   format!("presence:user:{}:conn:{}", user_id, conn_id)
}

#[inline]
pub fn presence_conn_count_key(user_id: i32) -> String {
   format!("presence:user:{}:conn_count", user_id)
}

#[inline]
pub fn presence_conn_pattern(user_id: i32) -> String {
   format!("presence:user:{}:conn:*", user_id)
}

#[inline]
pub fn user_events_topic(user_id: &str) -> String {
   format!("user:{}:events", user_id)
}

#[inline]
pub fn room_topic(room_id: &str) -> String {
   format!("room:{}", room_id)
}