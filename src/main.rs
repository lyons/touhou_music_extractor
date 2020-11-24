use std::{
  error::Error,
  path::{PathBuf},
  time::Duration,
};
use structopt::StructOpt;

mod bgminfo;
mod core;
mod wavheader;

use crate::core::{OutputOptions, OutputMode, LoopedFadeMode, extract_all};

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
    mode: OutputMode::Loops(1, LoopedFadeMode::After),
    fadeout_duration: 10,
  };

  //println!("Duration: {:?}", options.length);

  //Ok(())

  match bgm.game.pack_method {
    bgminfo::PackMethod::Two(_, _, _) => {
      extract_all(bgm, options.source, options.dest, &opts)
    },
    _ => {
      Err(format!("Unsupported pack method: {:?}", bgm.game.pack_method).into())
    },
  }
}
