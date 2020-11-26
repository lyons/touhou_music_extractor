use std::{
  error::Error,
  path::{PathBuf},
  time::Duration,
};
use structopt::StructOpt;

mod bgminfo;
mod core;
mod wavheader;

use crate::core::{OutputOptions, OutputMode, FadeMode};

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

  #[structopt(long, default_value = "{{name_jp}}/")]
  output_dir: String,
  #[structopt(long, default_value = "{{track_number}} - {{name_jp}}")]
  filename_format: String,

  #[structopt(parse(from_os_str))]
  bgminfo: PathBuf,
  #[structopt(parse(from_os_str), default_value = "thbgm.dat")]
  source: PathBuf,
}

fn main() -> Result<()> {
  let options = Options::from_args();
  
  let bgm = bgminfo::load_from_file(options.bgminfo)?;

  let opts =  OutputOptions {
    directory_format: options.output_dir,
    filename_format: options.filename_format,
    output_mode: OutputMode::FixedLoops(2, FadeMode::AfterLoopPoint),
    fadeout_duration: 10,
  };

  core::extract(&bgm, options.track_number, options.source, &opts)
}
