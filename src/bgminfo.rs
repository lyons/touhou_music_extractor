use regex::Regex;
use serde::{Serialize, Deserialize};
use std::{
  path::PathBuf,
};
use crate::Result;

// ---------------------------------------------------------------------------------------------------
// PUBLIC

#[derive(Debug, Clone, PartialEq)]
pub struct BgmInfo {
  pub game: Game,
  pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Game {
  pub name_jp: String,
  pub name_en: String,
  pub year: u32,
  pub game_number: String,
  pub artist: String,
  pub pack_method: PackMethod,
  pub tracks: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackMethod {
  One(String, u64),
  Two(String, u8, u8),
  Three(String, u64, Encryption),
  Four(String, u64, Encryption),
  Five(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Encryption {
  Simple(u64),
  MersenneTwister,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
  pub track_number: u32,

  pub start_offset: u64,
  pub relative_loop_offset: u64,
  pub relative_end_offset: u64,

  pub sample_rate: u32,

  pub name_jp: Option<String>,
  pub name_en: Option<String>,
  pub filename: Option<String>, // Not used with pack method 2
}

pub fn load(path: PathBuf) -> Result<BgmInfo> {
  let data = std::fs::read_to_string(path)?;
  let rewritten_data = rewrite_bgm_info(data);
  let raw_bgm: RawBgmInfo = toml::from_str(&rewritten_data)?;

  let game = bar(raw_bgm.game)?;
  let tracks = raw_bgm.tracks.into_iter().map(foo).collect::<Result<Vec<Track>>>()?;

  let result = BgmInfo {game, tracks};

  Ok(result)
}

// ---------------------------------------------------------------------------------------------------
// PRIVATE

#[derive(Debug, Serialize, Deserialize)]
struct RawBgmInfo {
  game: RawGame,
  tracks: Vec<RawTrack>,
}

fn default_header_size() -> u64 {
  0x2C
}

fn default_sample_rate() -> u32 {
  44100
}

#[derive(Debug, Serialize, Deserialize)]
struct RawGame {
  name: String,
  name_en: String,
  circle: Option<String>,
  circle_en: Option<String>,
  year: u32,
  gamenum: String,
  artist: String,
  artist_en: Option<String>,
  
  packmethod: u32,
  bgmdir: Option<String>, // Pack method 1
  bgmfile: Option<String>, // Pack methods 2, 3, 4, 5
  #[serde(default = "default_header_size")]
  headersize: u64, // Pack methods 1, 3, 4
  zwavid_08: Option<u8>, // Pack method 2
  zwavid_09: Option<u8>, // Pack method 2
  encryption: Option<u32>, // Pack method 3/4
  entrysize: Option<u64>, // Pack method 3/4 encryption method 1
  
  tracks: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawTrack {
  // Not present in original file, added by `rewrite_bgm_info()`
  track_number: u32,
  
  // `filename` must be present for pack methods 1, 3, 4, and 5
  name_jp: Option<String>,
  name_en: Option<String>,
  filename: Option<String>,
  
  // One of (position), (start, rel_loop, rel_end), (start, abs_loop, abs_end) must be present
  position: Option<Vec<u64>>,
  start: Option<u64>,
  rel_loop: Option<u64>,
  rel_end: Option<u64>,
  abs_loop: Option<u64>,
  abs_end: Option<u64>,

  #[serde(default = "default_sample_rate")]
  frequency: u32,
}

fn bar(game: RawGame) -> Result<Game> {
  let pack_method = match game.packmethod {
    1 => {
      if !game.bgmdir.is_some() { Err("Missing required field `bgmdir`") }
      else { Ok(PackMethod::One(game.bgmdir.unwrap(), game.headersize)) }
    },
    2 => {
      if !game.bgmfile.is_some() { Err("Missing required field `bgmfile`") }
      else if !game.zwavid_08.is_some() { Err("Missing required field `zwavid_08`") }
      else if !game.zwavid_09.is_some() { Err("Missing required field `zwavid_09`") }
      else {
        Ok(PackMethod::Two(game.bgmdir.unwrap(), game.zwavid_08.unwrap(), game.zwavid_09.unwrap()))
      }
    },
    _ => {
      Err("Unsupported pack method")
    },
  }?;

  let result = Game {
    name_jp: game.name,
    name_en: game.name_en,
    year: game.year,
    game_number: game.gamenum,
    artist: game.artist,
    pack_method,
    tracks: game.tracks,
  };

  Ok(result)
}

fn foo(track: RawTrack) -> Result<Track> {
  let start = if let Some(position) = track.position.clone() {
    position.get(0).map(|&n| n)
  }
  else {
    track.start
  }.ok_or_else(|| format!("Incomplete position data for track {}", track.track_number))?;

  let rel_loop = if let Some(position) = track.position.clone() {
    position.get(1).map(|&n| n)
  }
  else {
    if let Some(offset) = track.rel_loop { Some(offset) }
    else if let Some(offset) = track.abs_loop { Some(offset - start) }
    else { None }
  }.ok_or_else(|| format!("Incomplete position data for track {}", track.track_number))?;

  let rel_end = if let Some(position) = track.position.clone() {
    position.get(2).map(|&n| n)
  }
  else {
    if let Some(offset) = track.rel_end { Some(offset) }
    else if let Some(offset) = track.abs_end { Some(offset - start) }
    else { None }
  }.ok_or_else(|| format!("Incomplete position data for track {}", track.track_number))?;

  let result = Track {
    track_number: track.track_number,
    start_offset: start,
    relative_loop_offset: rel_loop,
    relative_end_offset: rel_end,
    sample_rate: track.frequency,
    name_jp: track.name_jp,
    name_en: track.name_en,
    filename: track.filename,
  };

  Ok(result)
}

fn rewrite_bgm_info(bgm: String) -> String {
  let track_number_re = Regex::new(r#"^\[(\d+)\]"#).unwrap();
  let position_re = Regex::new(r#"^position = "(.*)""#).unwrap();

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
