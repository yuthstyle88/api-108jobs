use app_108jobs_core::{
  error::{FastJobErrorType, FastJobResult},
  utils::slurs::check_slurs,
};
use regex::Regex;

pub mod admin;
pub mod category;
pub mod chat;
pub mod comment;
pub mod delivery;
pub mod local_user;
pub mod reports;
pub mod search;
pub mod site;
/// Check size of report
pub(crate) fn check_report_reason(reason: &str, slur_regex: &Regex) -> FastJobResult<()> {
  check_slurs(reason, slur_regex)?;
  if reason.is_empty() {
    Err(FastJobErrorType::ReportReasonRequired)?
  } else if reason.chars().count() > 1000 {
    Err(FastJobErrorType::ReportTooLong)?
  } else {
    Ok(())
  }
}
