use std::{
  collections::HashMap,
  error::Error,
  path::{PathBuf},
  time::Duration,
};
use string_template::Template;
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

  #[structopt(long, parse(from_os_str), default_value = "{{name_jp}}/")]
  output_dir: PathBuf,

  #[structopt(parse(from_os_str))]
  bgminfo: PathBuf,
  #[structopt(parse(from_os_str), default_value = "thbgm.dat")]
  source: PathBuf,
}

fn main() -> Result<()> {
  let options = Options::from_args();

  let output_dir = options.output_dir.to_str().unwrap();
  
  let bgm = bgminfo::load_from_file(options.bgminfo)?;

  let opts =  OutputOptions {
    mode: OutputMode::Loops(2, LoopedFadeMode::After),
    fadeout_duration: 10,
  };


  let template = Template::new(options.output_dir.to_str().unwrap());
  let mut args = HashMap::new();
  args.insert("name_jp", "東方紅魔郷　～ the Embodiment of Scarlet Devil");
  args.insert("name_en", "EoSD");

  let result = template.render(&args);
  println!("Path: {}", result);

  Ok(())

  //core::extract(bgm, options.track_number, options.source, dest, &opts)
}
