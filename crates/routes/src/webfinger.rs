use serde::Deserialize;

#[derive(Deserialize)]
struct Params {
  resource: String,
}
