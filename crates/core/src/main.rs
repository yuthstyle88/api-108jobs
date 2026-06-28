use cfg_if::cfg_if;

fn main() {
  cfg_if! {
    if #[cfg(feature = "full")] {
      println!("{}", config_to_string())
    } else {
    }
  }
}

#[cfg(feature = "full")]
fn config_to_string() -> String {
  use app_108jobs_core::settings::structs::Settings;
  use doku::json::{AutoComments, CommentsStyle, Formatting, ObjectsStyle};
  let fmt = Formatting {
    auto_comments: AutoComments::none(),
    comments_style: CommentsStyle {
      separator: "#".to_owned(),
    },
    objects_style: ObjectsStyle {
      surround_keys_with_quotes: false,
      use_comma_as_separator: false,
    },
    ..Default::default()
  };
  doku::to_json_fmt_val(&fmt, &Settings::default())
}
