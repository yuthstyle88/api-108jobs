use app_108jobs_core::{
  error::{FastJobErrorType, FastJobResult},
  utils::slurs::check_slurs,
};
use regex::Regex;

pub mod api;
pub mod crud;

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
