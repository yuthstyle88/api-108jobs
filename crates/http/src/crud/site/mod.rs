use app_108jobs_core::error::{FastJobErrorType, FastJobResult};
use app_108jobs_db::enums::{ListingType, RegistrationMode};

pub mod create;
pub mod read;
pub mod update;

pub fn site_default_post_listing_type_check(
  default_post_listing_type: &Option<ListingType>,
) -> FastJobResult<()> {
  if let Some(listing_type) = default_post_listing_type {
    if listing_type != &ListingType::All && listing_type != &ListingType::Local {
      Err(FastJobErrorType::InvalidDefaultPostListingType)?
    } else {
      Ok(())
    }
  } else {
    Ok(())
  }
}

pub fn application_question_check(
  current_application_question: &Option<String>,
  new_application_question: &Option<String>,
  registration_mode: RegistrationMode,
) -> FastJobResult<()> {
  let has_no_question: bool =
    current_application_question.is_none() && new_application_question.is_none();
  let is_nullifying_question: bool = new_application_question == &Some(String::new());

  if registration_mode == RegistrationMode::RequireApplication
    && (has_no_question || is_nullifying_question)
  {
    Err(FastJobErrorType::ApplicationQuestionRequired)?
  } else {
    Ok(())
  }
}

pub(super) fn not_zero(val: Option<i32>) -> Option<i32> {
  match val {
    Some(0) => None,
    v => v,
  }
}

#[cfg(test)]
mod tests {
  use super::{application_question_check, not_zero, site_default_post_listing_type_check};
  use app_108jobs_db::enums::{ListingType, RegistrationMode};

  #[test]
  fn test_site_default_post_listing_type_check() {
    assert!(site_default_post_listing_type_check(&None::<ListingType>).is_ok());
    assert!(site_default_post_listing_type_check(&Some(ListingType::All)).is_ok());
    assert!(site_default_post_listing_type_check(&Some(ListingType::Local)).is_ok());
    assert!(site_default_post_listing_type_check(&Some(ListingType::Subscribed)).is_err());
  }

  #[test]
  fn test_application_question_check() {
    assert!(application_question_check(
      &Some(String::from("q")),
      &Some(String::new()),
      RegistrationMode::RequireApplication
    )
    .is_err());
    assert!(
      application_question_check(&None, &None, RegistrationMode::RequireApplication).is_err()
    );
    assert!(application_question_check(&None, &None, RegistrationMode::Open).is_ok());
    assert!(application_question_check(
      &None,
      &Some(String::from("q")),
      RegistrationMode::RequireApplication
    )
    .is_ok());
    assert!(application_question_check(
      &Some(String::from("q")),
      &None,
      RegistrationMode::RequireApplication
    )
    .is_ok());
  }

  #[test]
  fn test_not_zero() {
    assert_eq!(None, not_zero(None));
    assert_eq!(None, not_zero(Some(0)));
    assert_eq!(Some(5), not_zero(Some(5)));
  }
}
