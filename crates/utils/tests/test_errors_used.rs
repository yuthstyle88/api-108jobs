use lemmy_utils::error::FastJobErrorType;
use std::{env::current_dir, process::Command};
use strum::IntoEnumIterator;

#[test]
#[allow(clippy::unwrap_used)]
fn test_errors_used() {
  let mut unused_error_found = false;
  let mut current_dir = current_dir().unwrap();
  current_dir.pop();
  current_dir.pop();
  for error in FastJobErrorType::iter() {
    let search = format!("FastJobErrorType::{error}");
    let mut grep_all = Command::new("grep");
    let grep_all = grep_all
      .current_dir(current_dir.clone())
      .arg("-R")
      .arg("--exclude=error.rs")
      .arg(&search)
      .arg("crates/")
      .arg("src/");
    let output = grep_all.output().unwrap();
    let grep_all_out = std::str::from_utf8(&output.stdout).unwrap();

    if grep_all_out.is_empty() {
      println!("FastJobErrorType::{} is unused", error);
      unused_error_found = true;
    }
  }
  assert!(!unused_error_found);
}
