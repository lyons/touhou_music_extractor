use regex::Regex;
use serde::{Serialize, Deserialize};
use crate::Result;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Game {
  pub name: String,
  pub name_en: String,
  pub circle: Option<String>,
  pub circle_en: Option<String>,
  pub year: Option<u32>,
  pub gamenum: Option<String>,
  pub packmethod: u32,
}

fn default_sample_rate() -> u32 {
  44100
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Track {
  pub track_number: u32,
  pub name_jp: String,
  pub name_en: String,
  pub position: Vec<u64>,
  #[serde(default = "default_sample_rate")]
  pub frequency: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BgmInfo {
  pub game: Game,
  pub tracks: Vec<Track>,
}

pub fn rewrite_bgm_info(bgm: String) -> String {
  let track_number_re = Regex::new(r#"^\[(\d+)\]"#).unwrap();
  let position_re = Regex::new(r#"^position = \"(.*)\""#).unwrap();

  let mut result = Vec::new();

  for line in bgm.split("\n") {
    if track_number_re.is_match(line) {
      result.push(String::from("[[tracks]]"));
      let temp = track_number_re.replace(line, "track_number = $1").into_owned();
      result.push(temp);
    }
    else if position_re.is_match(line) {
      let temp = position_re.replace(line, "position = [$1]").into_owned();
      result.push(temp);
    }
    else {
      result.push(line.to_string());
    }
  }

  result.join("\n")
}