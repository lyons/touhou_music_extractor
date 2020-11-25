use std::{
  error::Error,
  path::{PathBuf},
  time::Duration,
};
use structopt::StructOpt;
use tinytemplate::TinyTemplate;

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

  #[structopt(parse(from_os_str), default_value = "{name}/")]
  output_dir: PathBuf,

  #[structopt(parse(from_os_str))]
  bgminfo: PathBuf,
  #[structopt(parse(from_os_str), default_value = "thbgm.dat")]
  source: PathBuf,
}

fn main() -> Result<()> {
  let options = Options::from_args();

  let mut tt = TinyTemplate::new();
  let output_dir = options.output_dir.to_str().unwrap();
  tt.add_template("dest", output_dir);
  
  let bgm = bgminfo::load_from_file(options.bgminfo)?;

  let dest = tt.render("dest", &bgm.game)?;
  let dest = PathBuf::from(dest);

  let opts =  OutputOptions {
    mode: OutputMode::Loops(2, LoopedFadeMode::After),
    fadeout_duration: 10,
  };

  core::extract(bgm, options.track_number, options.source, dest, &opts)
}
