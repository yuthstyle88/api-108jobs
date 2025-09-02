use base64::{engine::general_purpose::STANDARD_NO_PAD as base64, Engine};
use captcha::Captcha;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{
  error::{FastJobErrorExt, FastJobErrorType, FastJobResult},
  utils::slurs::check_slurs,
};
use regex::Regex;
use std::io::Cursor;
use totp_rs::{Secret, TOTP};

pub mod admin;
pub mod comment;
pub mod community;
pub mod local_user;
pub mod post;
pub mod reports;
pub mod site;
pub mod chat;
/// Converts the captcha to a base64 encoded wav audio file
pub(crate) fn captcha_as_wav_base64(captcha: &Captcha) -> FastJobResult<String> {
  let letters = captcha.as_wav();

  // Decode each wav file, concatenate the samples
  let mut concat_samples: Vec<i16> = Vec::new();
  let mut any_header: Option<hound::WavSpec> = None;
  for letter in letters {
    let mut cursor = Cursor::new(letter.unwrap_or_default());
    let reader = hound::WavReader::new(&mut cursor)?;
    any_header = Some(reader.spec());
    let samples16 = reader
      .into_samples::<i16>()
      .collect::<Result<Vec<_>, _>>()
      .with_fastjob_type(FastJobErrorType::CouldntCreateAudioCaptcha)?;
    concat_samples.extend(samples16);
  }

  // Encode the concatenated result as a wav file
  let mut output_buffer = Cursor::new(vec![]);
  if let Some(header) = any_header {
    let mut writer = hound::WavWriter::new(&mut output_buffer, header)
      .with_fastjob_type(FastJobErrorType::CouldntCreateAudioCaptcha)?;
    let mut writer16 = writer.get_i16_writer(concat_samples.len().try_into()?);
    for sample in concat_samples {
      writer16.write_sample(sample);
    }
    writer16
      .flush()
      .with_fastjob_type(FastJobErrorType::CouldntCreateAudioCaptcha)?;
    writer
      .finalize()
      .with_fastjob_type(FastJobErrorType::CouldntCreateAudioCaptcha)?;

    Ok(base64.encode(output_buffer.into_inner()))
  } else {
    Err(FastJobErrorType::CouldntCreateAudioCaptcha)?
  }
}

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

pub(crate) fn check_totp_2fa_valid(
  local_user_view: &LocalUserView,
  totp_token: &Option<String>,
  site_name: &str,
) -> FastJobResult<()> {
  // Throw an error if their token is missing
  let token = totp_token
    .as_deref()
    .ok_or(FastJobErrorType::MissingTotpToken)?;
  let secret = local_user_view
    .local_user
    .totp_2fa_secret
    .as_deref()
    .ok_or(FastJobErrorType::MissingTotpSecret)?;

  let totp = build_totp_2fa(site_name, &local_user_view.person.name, secret)?;

  let check_passed = totp.check_current(token)?;
  if !check_passed {
    return Err(FastJobErrorType::IncorrectTotpToken.into());
  }

  Ok(())
}

pub(crate) fn generate_totp_2fa_secret() -> String {
  Secret::generate_secret().to_string()
}

fn build_totp_2fa(hostname: &str, username: &str, secret: &str) -> FastJobResult<TOTP> {
  let sec = Secret::Raw(secret.as_bytes().to_vec());
  let sec_bytes = sec
    .to_bytes()
    .with_fastjob_type(FastJobErrorType::CouldntParseTotpSecret)?;

  TOTP::new(
    totp_rs::Algorithm::SHA1,
    6,
    1,
    30,
    sec_bytes,
    Some(hostname.to_string()),
    username.to_string(),
  )
  .with_fastjob_type(FastJobErrorType::CouldntGenerateTotp)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_build_totp() {
    let generated_secret = generate_totp_2fa_secret();
    let totp = build_totp_2fa("lemmy.ml", "my_name", &generated_secret);
    assert!(totp.is_ok());
  }
}
