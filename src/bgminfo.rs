use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Game {
  pub name: String,
  pub name_en: String,
  pub gamenum: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Track {
  pub track_number: u32,
  pub name_jp: String,
  pub name_en: String,
  pub position: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BgmInfo {
  pub game: Game,
  pub tracks: Vec<Track>,
}