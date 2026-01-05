use assert_json_diff::assert_json_include;
use app_108jobs_utils::error::FastJobResult;
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, fs::File, io::BufReader};

pub fn file_to_json_object<T: DeserializeOwned>(path: &str) -> FastJobResult<T> {
  let file = File::open(path)?;
  let reader = BufReader::new(file);
  Ok(serde_json::from_reader(reader)?)
}


/// Check that json deserialize -> serialize -> deserialize gives identical file as initial one.
/// Ensures that there are no breaking changes in sent data.
pub fn test_parse_app_108jobs_item<T: Serialize + DeserializeOwned + std::fmt::Debug>(
  path: &str,
) -> FastJobResult<T> {
  // parse file as T
  let parsed = file_to_json_object::<T>(path)?;

  // parse file into hashmap, which ensures that every field is included
  let raw = file_to_json_object::<HashMap<String, serde_json::Value>>(path)?;
  // assert that all fields are identical, otherwise print diff
  assert_json_include!(actual: &parsed, expected: raw);
  Ok(parsed)
}

