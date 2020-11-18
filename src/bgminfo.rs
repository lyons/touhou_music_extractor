use serde::{Serialize, Deserialize};

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