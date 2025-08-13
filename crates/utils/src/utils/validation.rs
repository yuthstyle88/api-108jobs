use crate::error::{FastJobError, FastJobErrorExt, FastJobErrorType, FastJobResult, MAX_API_PARAM_ELEMENTS};
use clearurls::UrlCleaner;
use diesel::internal::derives::multiconnection::chrono::NaiveDate;
use invisible_characters::INVISIBLE_CHARS;
use itertools::Itertools;
use regex::{Regex, RegexBuilder, RegexSet};
use std::sync::LazyLock;
use unicode_segmentation::UnicodeSegmentation;
use url::{ParseError, Url};

// From here: https://github.com/vector-im/element-android/blob/develop/matrix-sdk-android/src/main/java/org/matrix/android/sdk/api/MatrixPatterns.kt#L35
#[allow(clippy::expect_used)]
static VALID_MATRIX_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^@[A-Za-z0-9\x21-\x39\x3B-\x7F]+:[A-Za-z0-9.-]+(:[0-9]{2,5})?$")
    .expect("compile regex")
});
// taken from https://en.wikipedia.org/wiki/UTM_parameters
#[allow(clippy::expect_used)]
static URL_CLEANER: LazyLock<UrlCleaner> =
  LazyLock::new(|| UrlCleaner::from_embedded_rules().expect("compile clearurls"));
const ALLOWED_POST_URL_SCHEMES: [&str; 3] = ["http", "https", "magnet"];

const BODY_MAX_LENGTH: usize = 10000;
const POST_BODY_MAX_LENGTH: usize = 50000;
const BIO_MAX_LENGTH: usize = 1000;
const URL_MAX_LENGTH: usize = 2000;
const ALT_TEXT_MAX_LENGTH: usize = 1500;
const SITE_NAME_MAX_LENGTH: usize = 20;
const SITE_NAME_MIN_LENGTH: usize = 1;
const SITE_DESCRIPTION_MAX_LENGTH: usize = 150;
const MIN_LENGTH_BLOCKING_KEYWORD: usize = 3;
const MAX_LENGTH_BLOCKING_KEYWORD: usize = 50;
const TAG_NAME_MIN_LENGTH: usize = 3;
const TAG_NAME_MAX_LENGTH: usize = 100;

fn has_newline(name: &str) -> bool {
  name.contains('\n')
}

pub fn is_valid_actor_name(name: &str, actor_name_max_length: i32) -> FastJobResult<()> {
  // Only allow characters from a single alphabet per username. This avoids problems with lookalike
  // characters like `o` which looks identical in Latin and Cyrillic, and can be used to imitate
  // other users. Checks for additional alphabets can be added in the same way.
  #[allow(clippy::expect_used)]
  static VALID_ACTOR_NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:[a-zA-Z0-9_]+|[0-9_\p{Arabic}]+|[0-9_\p{Cyrillic}]+|[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})$")
     .expect("compile regex")
  });

  let actor_name_max_length: usize = actor_name_max_length.try_into()?;
  min_length_check(name, 3, FastJobErrorType::InvalidName)?;
  max_length_check(name, actor_name_max_length, FastJobErrorType::InvalidName)?;
  if VALID_ACTOR_NAME_REGEX.is_match(name) {
    Ok(())
  } else {
    Err(FastJobErrorType::InvalidName.into())
  }
}

fn has_3_permitted_display_chars(name: &str) -> bool {
  let mut num_non_fdc: i8 = 0;
  for c in name.chars() {
    if !INVISIBLE_CHARS.contains(&c) {
      num_non_fdc += 1;
      if num_non_fdc >= 3 {
        break;
      }
    }
  }
  if num_non_fdc >= 3 {
    return true;
  }
  false
}

// Can't do a regex here, reverse lookarounds not supported
pub fn is_valid_display_name(name: &str, actor_name_max_length: i32) -> FastJobResult<()> {
  let actor_name_max_length: usize = actor_name_max_length.try_into()?;
  let check = !name.starts_with('@')
    && !name.starts_with(INVISIBLE_CHARS)
    && name.chars().count() <= actor_name_max_length
    && !has_newline(name)
    && has_3_permitted_display_chars(name);
  if !check {
    Err(FastJobErrorType::InvalidDisplayName.into())
  } else {
    Ok(())
  }
}

pub fn is_valid_matrix_id(matrix_id: &str) -> FastJobResult<()> {
  let check = VALID_MATRIX_ID_REGEX.is_match(matrix_id) && !has_newline(matrix_id);
  if !check {
    Err(FastJobErrorType::InvalidMatrixId.into())
  } else {
    Ok(())
  }
}

pub fn is_valid_post_title(title: &str) -> FastJobResult<()> {
  let length = title.trim().chars().count();
  let check =
    (3..=200).contains(&length) && !has_newline(title) && has_3_permitted_display_chars(title);
  if !check {
    Err(FastJobErrorType::InvalidPostTitle.into())
  } else {
    Ok(())
  }
}

/// This could be post bodies, comments, notes, or any description field
pub fn is_valid_body_field(body: &str, post: bool) -> FastJobResult<()> {
  if post {
    max_length_check(body, POST_BODY_MAX_LENGTH, FastJobErrorType::InvalidBodyField)?;
  } else {
    max_length_check(body, BODY_MAX_LENGTH, FastJobErrorType::InvalidBodyField)?;
  };
  Ok(())
}

pub fn is_valid_bio_field(bio: &str) -> FastJobResult<()> {
  max_length_check(bio, BIO_MAX_LENGTH, FastJobErrorType::BioLengthOverflow)
}

pub fn is_valid_alt_text_field(alt_text: &str) -> FastJobResult<()> {
  max_length_check(
    alt_text,
    ALT_TEXT_MAX_LENGTH,
    FastJobErrorType::AltTextLengthOverflow,
  )?;

  Ok(())
}

/// Checks the site name length, the limit as defined in the DB.
pub fn site_name_length_check(name: &str) -> FastJobResult<()> {
  min_length_check(name, SITE_NAME_MIN_LENGTH, FastJobErrorType::SiteNameRequired)?;
  max_length_check(
    name,
    SITE_NAME_MAX_LENGTH,
    FastJobErrorType::SiteNameLengthOverflow,
  )
}

/// Checks the site / community description length, the limit as defined in the DB.
pub fn site_or_community_description_length_check(description: &str) -> FastJobResult<()> {
  max_length_check(
    description,
    SITE_DESCRIPTION_MAX_LENGTH,
    FastJobErrorType::SiteDescriptionLengthOverflow,
  )
}

pub fn tag_name_length_check(tag_name: &str) -> FastJobResult<()> {
  min_length_check(
    tag_name,
    TAG_NAME_MIN_LENGTH,
    FastJobErrorType::InvalidTagName,
  )?;
  max_length_check(
    tag_name,
    TAG_NAME_MAX_LENGTH,
    FastJobErrorType::InvalidTagName,
  )
}

/// Check minimum and maximum length of input string. If the string is too short or too long, the
/// corresponding error is returned.
///
/// HTML frontends specify maximum input length using `maxlength` attribute.
/// For consistency we use the same counting method (UTF-16 code units).
/// https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes/maxlength
fn max_length_check(item: &str, max_length: usize, max_msg: FastJobErrorType) -> FastJobResult<()> {
  let len = item.encode_utf16().count();
  if len > max_length {
    Err(max_msg.into())
  } else {
    Ok(())
  }
}

fn min_length_check(item: &str, min_length: usize, min_msg: FastJobErrorType) -> FastJobResult<()> {
  let len = item.encode_utf16().count();
  if len < min_length {
    Err(min_msg.into())
  } else {
    Ok(())
  }
}

/// Attempts to build a regex and check it for common errors before inserting into the DB.
pub fn build_and_check_regex(regex_str_opt: Option<&str>) -> FastJobResult<Regex> {
  // Placeholder regex which doesnt match anything
  // https://stackoverflow.com/a/940840
  let match_nothing = RegexBuilder::new("a^")
    .build()
    .with_fastjob_type(FastJobErrorType::InvalidRegex);
  if let Some(regex) = regex_str_opt {
    if regex.is_empty() {
      match_nothing
    } else {
      RegexBuilder::new(regex)
        .case_insensitive(true)
        .build()
        .with_fastjob_type(FastJobErrorType::InvalidRegex)
        .and_then(|regex| {
          // NOTE: It is difficult to know, in the universe of user-crafted regex, which ones
          // may match against any string text. To keep it simple, we'll match the regex
          // against an innocuous string - a single number - which should help catch a regex
          // that accidentally matches against all strings.
          if regex.is_match("1") {
            Err(FastJobErrorType::PermissiveRegex.into())
          } else {
            Ok(regex)
          }
        })
    }
  } else {
    match_nothing
  }
}

/// Cleans a url of tracking parameters.
pub fn clean_url(url: &Url) -> Url {
  match URL_CLEANER.clear_single_url(url) {
    Ok(res) => res.into_owned(),
    // If there are any errors, just return the original url
    Err(_) => url.clone(),
  }
}

/// Cleans all the links in a string of tracking parameters.
pub fn clean_urls_in_text(text: &str) -> String {
  match URL_CLEANER.clear_text(text) {
    Ok(res) => res.into_owned(),
    // If there are any errors, just return the original text
    Err(_) => text.to_owned(),
  }
}

pub fn is_valid_url(url: &Url) -> FastJobResult<()> {
  if !ALLOWED_POST_URL_SCHEMES.contains(&url.scheme()) {
    Err(FastJobErrorType::InvalidUrlScheme)?
  }

  max_length_check(
    url.as_str(),
    URL_MAX_LENGTH,
    FastJobErrorType::UrlLengthOverflow,
  )?;

  Ok(())
}

pub fn is_url_blocked(url: &Url, blocklist: &RegexSet) -> FastJobResult<()> {
  if blocklist.is_match(url.as_str()) {
    Err(FastJobErrorType::BlockedUrl)?
  }

  Ok(())
}

/// Check that urls are valid, and also remove the scheme, and uniques
pub fn check_urls_are_valid(urls: &Vec<String>) -> FastJobResult<Vec<String>> {
  let mut parsed_urls = vec![];
  for url in urls {
    parsed_urls.push(build_url_str_without_scheme(url)?);
  }

  let unique_urls = parsed_urls.into_iter().unique().collect();
  Ok(unique_urls)
}

pub fn check_blocking_keywords_are_valid(blocking_keywords: &Vec<String>) -> FastJobResult<()> {
  for keyword in blocking_keywords {
    min_length_check(
      keyword,
      MIN_LENGTH_BLOCKING_KEYWORD,
      FastJobErrorType::BlockKeywordTooShort,
    )?;
    max_length_check(
      keyword,
      MAX_LENGTH_BLOCKING_KEYWORD,
      FastJobErrorType::BlockKeywordTooLong,
    )?;
  }
  check_api_elements_count(blocking_keywords.len())?;
  Ok(())
}

pub fn build_url_str_without_scheme(url_str: &str) -> FastJobResult<String> {
  // Parse and check for errors
  let mut url = Url::parse(url_str).or_else(|e| {
    if e == ParseError::RelativeUrlWithoutBase {
      Url::parse(&format!("http://{url_str}"))
    } else {
      Err(e)
    }
  })?;

  // Set the scheme to http, then remove the http:// part
  url
    .set_scheme("http")
    .map_err(|_e| FastJobErrorType::InvalidUrl)?;

  let mut out = url
    .to_string()
    .get(7..)
    .ok_or(FastJobErrorType::InvalidUrl)?
    .to_string();

  // Remove trailing / if necessary
  if out.ends_with('/') {
    out.pop();
  }

  Ok(out)
}

// Shorten a string to n chars, being mindful of unicode grapheme
// boundaries
pub fn truncate_for_db(text: &str, len: usize) -> String {
  if text.chars().count() <= len {
    text.to_string()
  } else {
    let offset = text
      .char_indices()
      .nth(len)
      .unwrap_or(text.char_indices().last().unwrap_or_default());
    let graphemes: Vec<(usize, _)> = text.grapheme_indices(true).collect();
    let mut index = 0;
    // Walk the string backwards and find the first char within our length
    for idx in (0..graphemes.len()).rev() {
      if let Some(grapheme) = graphemes.get(idx) {
        if grapheme.0 < offset.0 {
          index = idx;
          break;
        }
      }
    }
    let grapheme = graphemes.get(index).unwrap_or(&(0, ""));
    let grapheme_count = grapheme.1.chars().count();
    // `take` isn't inclusive, so if the last grapheme can fit we add its char
    // length
    let char_count = if grapheme_count + grapheme.0 <= len {
      grapheme.0 + grapheme_count
    } else {
      grapheme.0
    };

    text.chars().take(char_count).collect::<String>()
  }
}

pub fn truncate_description(text: &str) -> String {
  truncate_for_db(text, SITE_DESCRIPTION_MAX_LENGTH)
}

pub fn check_api_elements_count(len: usize) -> FastJobResult<()> {
  if len >= MAX_API_PARAM_ELEMENTS {
    Err(FastJobErrorType::TooManyItems)?
  }
  Ok(())
}

/// Checks the password length
pub fn password_length_check(pass: &str) -> FastJobResult<()> {
  if !(6..=60).contains(&pass.chars().count()) {
    Err(FastJobErrorType::InvalidPassword)?
  } else {
    Ok(())
  }
}

pub fn is_valid_email(email: &str) -> bool {
  // This is a reasonable basic email regex
  let email_re = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
  email_re.is_match(email)
}

pub fn is_valid_phone(phone: &str) -> bool {
  // Basic phone validation: only contains digits, +, -, and spaces
  // and has at least 7 digits
  let digit_count = phone.chars().filter(|c| c.is_digit(10)).count();
  let valid_chars = phone.chars().all(|c| c.is_digit(10) || c == '+' || c == '-' || c == ' ');

  valid_chars && digit_count >= 7
}
pub fn is_valid_issued_and_expiry(issued: Option<NaiveDate>, expiry: Option<NaiveDate>) -> bool {
  match (issued, expiry) {
    (Some(issued), Some(expiry)) => expiry > issued,
    _ => true, // ถ้าไม่มีวันใดวันหนึ่ง ถือว่าผ่าน
  }
}
pub fn get_required_trimmed(
  field: &Option<String>,
  err: FastJobErrorType,
) -> Result<String, FastJobError> {
  field
      .as_ref()
      .map(|s| s.trim().to_owned())
      .filter(|s| !s.is_empty())
      .ok_or_else(|| err.into())
}

// National ID validation
pub trait NationalIdValidator {
  fn is_valid(&self, id: &str) -> bool;
  fn country_name(&self) -> &'static str;
}

pub struct ThaiIdValidator;

impl NationalIdValidator for ThaiIdValidator {
  fn is_valid(&self, id: &str) -> bool {
    if id.len() != 13 || !id.chars().all(|c| c.is_ascii_digit()) {
      return false;
    }

    let digits: Vec<u32> = id.chars().filter_map(|c| c.to_digit(10)).collect();

    let sum: u32 = digits.iter()
        .take(12)
        .enumerate()
        .map(|(i, &digit)| digit * (13 - i as u32))
        .sum();

    let check_digit = (11 - (sum % 11)) % 10;
    check_digit == digits[12]
  }

  fn country_name(&self) -> &'static str {
    "Thailand"
  }
}

pub struct VietnamIdValidator;

impl NationalIdValidator for VietnamIdValidator {
  fn is_valid(&self, id: &str) -> bool {
    // Must be either 9 or 12 digits
    matches!(id.len(), 9 | 12) && id.chars().all(|c| c.is_ascii_digit())
  }

  fn country_name(&self) -> &'static str {
    "Vietnam"
  }
}
pub fn validate_bank_account(country_id: &str, account_number: &str) -> bool {
  let num = account_number.trim();

  match country_id.to_uppercase().as_str() {
    // Thailand: usually 10-12 digits, all numeric
    "TH" => {
      num.len() >= 10
      && num.len() <= 12
      && num.chars().all(|c| c.is_ascii_digit())
    }
    // Vietnam: usually 8-15 digits, all numeric
    "VN" => {
      num.len() >= 8
      && num.len() <= 15
      && num.chars().all(|c| c.is_ascii_digit())
    }
    // Unsupported country
    _ => false,
  }
}

pub fn validate_amount(amount: &f64) -> FastJobResult<()> {
  if amount.is_sign_negative() {
    return Err(FastJobErrorType::NegativeAmount.into());
  }

  Ok(())
}

#[inline]
pub fn round_to_2_decimals(amount: f64) -> f64 {
  (amount * 100.0).round() / 100.0
}
#[cfg(test)]
mod tests {
  use crate::{
    error::{FastJobErrorType, FastJobResult},
    utils::validation::{
      build_and_check_regex,
      check_urls_are_valid,
      clean_url,
      clean_urls_in_text,
      is_url_blocked,
      is_valid_actor_name,
      is_valid_bio_field,
      is_valid_display_name,
      is_valid_matrix_id,
      is_valid_post_title,
      is_valid_url,
      site_name_length_check,
      site_or_community_description_length_check,
      truncate_for_db,
      BIO_MAX_LENGTH,
      SITE_DESCRIPTION_MAX_LENGTH,
      SITE_NAME_MAX_LENGTH,
      URL_MAX_LENGTH,
    },
  };
  use pretty_assertions::assert_eq;
  use url::Url;

  const URL_WITH_TRACKING: &str = "https://example.com/path/123?utm_content=buffercf3b2&utm_medium=social&user+name=random+user&id=123";
  const URL_TRACKING_REMOVED: &str = "https://example.com/path/123?user+name=random+user&id=123";

  #[test]
  fn test_clean_url_params() -> FastJobResult<()> {
    let url = Url::parse(URL_WITH_TRACKING)?;
    let cleaned = clean_url(&url);
    let expected = Url::parse(URL_TRACKING_REMOVED)?;
    assert_eq!(expected.to_string(), cleaned.to_string());

    let url = Url::parse("https://example.com/path/123")?;
    let cleaned = clean_url(&url);
    assert_eq!(url.to_string(), cleaned.to_string());

    Ok(())
  }

  #[test]
  fn test_clean_body() -> FastJobResult<()> {
    let text = format!("[a link]({URL_WITH_TRACKING})");
    let cleaned = clean_urls_in_text(&text);
    let expected = format!("[a link]({URL_TRACKING_REMOVED})");
    assert_eq!(expected.to_string(), cleaned.to_string());

    let text = "[a link](https://example.com/path/123)";
    let cleaned = clean_urls_in_text(text);
    assert_eq!(text.to_string(), cleaned);

    Ok(())
  }

  #[test]
  fn regex_checks() {
    assert!(is_valid_post_title("hi").is_err());
    assert!(is_valid_post_title("him").is_ok());
    assert!(is_valid_post_title("  him  ").is_ok());
    assert!(is_valid_post_title("n\n\n\n\nanother").is_err());
    assert!(is_valid_post_title("hello there!\n this is a test.").is_err());
    assert!(is_valid_post_title("hello there! this is a test.").is_ok());
    assert!(is_valid_post_title(("12345".repeat(40) + "x").as_str()).is_err());
    assert!(is_valid_post_title("12345".repeat(40).as_str()).is_ok());
    assert!(is_valid_post_title((("12345".repeat(40)) + "  ").as_str()).is_ok());
  }

  #[test]
  fn test_valid_actor_name() {
    let actor_name_max_length = 20;
    assert!(is_valid_actor_name("Hello_98", actor_name_max_length).is_ok());
    assert!(is_valid_actor_name("ten", actor_name_max_length).is_ok());
    assert!(is_valid_actor_name("تجريب", actor_name_max_length).is_ok());
    assert!(is_valid_actor_name("تجريب_123", actor_name_max_length).is_ok());
    assert!(is_valid_actor_name("Владимир", actor_name_max_length).is_ok());

    // mixed scripts
    assert!(is_valid_actor_name("تجريب_abc", actor_name_max_length).is_err());
    assert!(is_valid_actor_name("Влад_abc", actor_name_max_length).is_err());
    // dash
    assert!(is_valid_actor_name("Hello-98", actor_name_max_length).is_err());
    // too short
    assert!(is_valid_actor_name("a", actor_name_max_length).is_err());
    // empty
    assert!(is_valid_actor_name("", actor_name_max_length).is_err());
    // newline
    assert!(is_valid_actor_name(
      r"Line1

Line3",
      actor_name_max_length
    )
    .is_err());
    assert!(is_valid_actor_name("Line1\nLine3", actor_name_max_length).is_err());
  }

  #[test]
  fn test_valid_display_name() {
    let actor_name_max_length = 20;
    assert!(is_valid_display_name("hello @there", actor_name_max_length).is_ok());
    assert!(is_valid_display_name("@hello there", actor_name_max_length).is_err());
    assert!(is_valid_display_name("\u{200d}hello", actor_name_max_length).is_err());
    assert!(is_valid_display_name(
      "\u{1f3f3}\u{fe0f}\u{200d}\u{26a7}\u{fe0f}Name",
      actor_name_max_length
    )
    .is_ok());
    assert!(is_valid_display_name("\u{2003}1\u{ffa0}2\u{200d}", actor_name_max_length).is_err());

    // Make sure zero-space with an @ doesn't work
    assert!(
      is_valid_display_name(&format!("{}@my name is", '\u{200b}'), actor_name_max_length).is_err()
    );
  }

  #[test]
  fn test_valid_post_title() {
    assert!(is_valid_post_title("Post Title").is_ok());
    assert!(is_valid_post_title(
      "აშშ ითხოვს ირანს დაუყოვნებლივ გაანთავისუფლოს დაკავებული ნავთობის ტანკერი"
    )
    .is_ok());
    assert!(is_valid_post_title("   POST TITLE 😃😃😃😃😃").is_ok());
    assert!(is_valid_post_title("\n \n \n \n    		").is_err()); // tabs/spaces/newlines
    assert!(is_valid_post_title("\u{206a}").is_err()); // invisible chars
    assert!(is_valid_post_title("\u{1f3f3}\u{fe0f}\u{200d}\u{26a7}\u{fe0f}").is_ok());
  }

  #[test]
  fn test_valid_matrix_id() {
    assert!(is_valid_matrix_id("@dess:matrix.org").is_ok());
    assert!(is_valid_matrix_id("@dess_:matrix.org").is_ok());
    assert!(is_valid_matrix_id("@dess:matrix.org:443").is_ok());
    assert!(is_valid_matrix_id("dess:matrix.org").is_err());
    assert!(is_valid_matrix_id(" @dess:matrix.org").is_err());
    assert!(is_valid_matrix_id("@dess:matrix.org t").is_err());
    assert!(is_valid_matrix_id("@dess:matrix.org t").is_err());
  }

  #[test]
  fn test_valid_site_name() -> FastJobResult<()> {
    let valid_names = [
      (0..SITE_NAME_MAX_LENGTH).map(|_| 'A').collect::<String>(),
      String::from("A"),
    ];
    let invalid_names = [
      (
        &(0..SITE_NAME_MAX_LENGTH + 1)
          .map(|_| 'A')
          .collect::<String>(),
        FastJobErrorType::SiteNameLengthOverflow,
      ),
      (&String::new(), FastJobErrorType::SiteNameRequired),
    ];

    valid_names.iter().for_each(|valid_name| {
      assert!(
        site_name_length_check(valid_name).is_ok(),
        "Expected {} of length {} to be Ok.",
        valid_name,
        valid_name.len()
      )
    });

    invalid_names
      .iter()
      .for_each(|(invalid_name, expected_err)| {
        let result = site_name_length_check(invalid_name);

        assert!(result.is_err());
        assert!(
          result.is_err_and(|e| e.error_type.eq(&expected_err.clone())),
          "Testing {}, expected error {}",
          invalid_name,
          expected_err
        );
      });
    Ok(())
  }

  #[test]
  fn test_valid_bio() {
    assert!(is_valid_bio_field(&(0..BIO_MAX_LENGTH).map(|_| 'A').collect::<String>()).is_ok());

    let invalid_result =
      is_valid_bio_field(&(0..BIO_MAX_LENGTH + 1).map(|_| 'A').collect::<String>());

    assert!(
      invalid_result.is_err()
        && invalid_result.is_err_and(|e| e.error_type.eq(&FastJobErrorType::BioLengthOverflow))
    );
  }

  #[test]
  fn test_valid_site_description() {
    assert!(site_or_community_description_length_check(
      &(0..SITE_DESCRIPTION_MAX_LENGTH)
        .map(|_| 'A')
        .collect::<String>()
    )
    .is_ok());

    let invalid_result = site_or_community_description_length_check(
      &(0..SITE_DESCRIPTION_MAX_LENGTH + 1)
        .map(|_| 'A')
        .collect::<String>(),
    );

    assert!(
      invalid_result.is_err()
        && invalid_result.is_err_and(|e| e
          .error_type
          .eq(&FastJobErrorType::SiteDescriptionLengthOverflow))
    );
  }

  #[test]
  fn test_valid_slur_regex() -> FastJobResult<()> {
    let valid_regex = Some("(foo|bar)");
    build_and_check_regex(valid_regex)?;

    let missing_regex = None;
    let match_none = build_and_check_regex(missing_regex)?;
    assert!(!match_none.is_match(""));
    assert!(!match_none.is_match("a"));

    let empty = Some("");
    let match_none = build_and_check_regex(empty)?;
    assert!(!match_none.is_match(""));
    assert!(!match_none.is_match("a"));

    Ok(())
  }

  #[test]
  fn test_too_permissive_slur_regex() {
    let match_everything_regexes = [
      (Some("["), FastJobErrorType::InvalidRegex),
      (Some("(foo|bar|)"), FastJobErrorType::PermissiveRegex),
      (Some(".*"), FastJobErrorType::PermissiveRegex),
    ];

    match_everything_regexes
      .into_iter()
      .for_each(|(regex_str, expected_err)| {
        let result = build_and_check_regex(regex_str);

        assert!(result.is_err());
        assert!(
          result.is_err_and(|e| e.error_type.eq(&expected_err.clone())),
          "Testing regex {:?}, expected error {}",
          regex_str,
          expected_err
        );
      });
  }

  #[test]
  fn test_check_url_valid() -> FastJobResult<()> {
    assert!(is_valid_url(&Url::parse("http://example.com")?).is_ok());
    assert!(is_valid_url(&Url::parse("https://example.com")?).is_ok());
    assert!(is_valid_url(&Url::parse("https://example.com")?).is_ok());
    assert!(is_valid_url(&Url::parse("ftp://example.com")?)
      .is_err_and(|e| e.error_type.eq(&FastJobErrorType::InvalidUrlScheme)));
    assert!(is_valid_url(&Url::parse("javascript:void")?)
      .is_err_and(|e| e.error_type.eq(&FastJobErrorType::InvalidUrlScheme)));

    let magnet_link="magnet:?xt=urn:btih:4b390af3891e323778959d5abfff4b726510f14c&dn=Ravel%20Complete%20Piano%20Sheet%20Music%20-%20Public%20Domain&tr=udp%3A%2F%2Fopen.tracker.cl%3A1337%2Fannounce";
    assert!(is_valid_url(&Url::parse(magnet_link)?).is_ok());

    // Also make sure the length overflow hits an error
    let mut long_str = "http://example.com/test=".to_string();
    for _ in 1..URL_MAX_LENGTH {
      long_str.push('X');
    }
    let long_url = Url::parse(&long_str)?;
    assert!(
      is_valid_url(&long_url).is_err_and(|e| e.error_type.eq(&FastJobErrorType::UrlLengthOverflow))
    );

    Ok(())
  }

  #[test]
  fn test_url_block() -> FastJobResult<()> {
    let set = regex::RegexSet::new(vec![
      r"(https://)?example\.org/page/to/article",
      r"(https://)?example\.net/?",
      r"(https://)?example\.com/?",
    ])?;

    assert!(is_url_blocked(&Url::parse("https://example.blog")?, &set).is_ok());

    assert!(is_url_blocked(&Url::parse("https://example.org")?, &set).is_ok());

    assert!(is_url_blocked(&Url::parse("https://example.com")?, &set).is_err());

    Ok(())
  }

  #[test]
  fn test_url_parsed() -> FastJobResult<()> {
    // Make sure the scheme is removed, and uniques also
    assert_eq!(
      &check_urls_are_valid(&vec![
        "example.com".to_string(),
        "http://example.com".to_string(),
        "https://example.com".to_string(),
        "https://example.com/test?q=test2&q2=test3#test4".to_string(),
      ])?,
      &vec![
        "example.com".to_string(),
        "example.com/test?q=test2&q2=test3#test4".to_string()
      ],
    );

    assert!(check_urls_are_valid(&vec!["https://example .com".to_string()]).is_err());
    Ok(())
  }

  #[test]
  fn test_truncate() -> FastJobResult<()> {
    assert_eq!("Hell", truncate_for_db("Hello", 4));
    assert_eq!("word", truncate_for_db("word", 10));
    assert_eq!("Wales: ", truncate_for_db("Wales: 🏴󠁧󠁢󠁷󠁬󠁳󠁿", 10));
    assert_eq!("Wales: 🏴󠁧󠁢󠁷󠁬󠁳󠁿", truncate_for_db("Wales: 🏴󠁧󠁢󠁷󠁬󠁳󠁿", 14));

    Ok(())
  }
}
