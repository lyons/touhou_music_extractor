use std::{
  error::Error,
  path::{PathBuf},
  time::Duration,
};
use structopt::StructOpt;

mod bgminfo;
mod core;
mod wavheader;

use crate::core::{OutputOptions, OutputMode, LoopedFadeMode};

pub(crate) type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(StructOpt)]
struct Options {
  /// Length of output file
  ///
  /// Blah blah blah
  // #[structopt(long, parse(try_from_str = parse_duration::parse))]
  // length: Option<Duration>,
  // #[structopt(long, conflicts_with("length"))]
  // loops: Option<u32>,
  // #[structopt(long, parse(try_from_str = parse_duration::parse), default_value = "10")]
  // fadeout_length: Duration,

  #[structopt(long)]
  track_number: Option<usize>,

  #[structopt(parse(from_os_str))]
  bgminfo: PathBuf,
  #[structopt(parse(from_os_str), default_value = "thbgm.dat")]
  source: PathBuf,
  #[structopt(parse(from_os_str), default_value = "output/")]
  dest: PathBuf,
}

fn main() -> Result<()> {
  let options = Options::from_args();
  
  let bgm = bgminfo::load(options.bgminfo)?;

  let opts =  OutputOptions {
    mode: OutputMode::Loops(2, LoopedFadeMode::After),
    fadeout_duration: 10,
  };

  core::extract(bgm, options.track_number, options.source, options.dest, &opts)
}
