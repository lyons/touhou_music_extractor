#![forbid(unsafe_code)]

use std::{
  convert::TryFrom,
  error::Error,
  path::{PathBuf},
  time::Duration,
};
use structopt::StructOpt;

mod bgminfo;
mod bgmstore;
mod core;
mod wavheader;

use crate::{
  bgminfo::BgmInfo,
  bgmstore::BgmStore,
  core::{OutputOptions, OutputMode, FadeMode},
};

pub(crate) type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

/// Foo bar baz
#[derive(StructOpt)]
struct Options {
  #[structopt(subcommand)]
  mode: Command,

  #[structopt(
    short = "n",
    long = "--track",
    global = true,
    display_order = 1,
    help = "When provided, extracts a specified single track; otherwise, all tracks are extracted",
    long_help = "When provided, extracts a specified single track; otherwise, all tracks are extracted."
  )]
  track_number: Option<usize>,
  
  #[structopt(
    long = "--fadeout-length",
    value_name = "duration",
    parse(try_from_str = parse_duration::parse),
    global = true,
    display_order = 4,
    help = "Length in seconds of fadeout at the end of each track",
    long_help =
r"Length during which the end of each track should be faded out. Fadeout duration will be capped to a maximum of one loop length of the track. Set to `0` for no fadeout.

Value can be given as an integer value (in seconds), or as a duration with `h`, `m`, and `s` used to indicate hours, minutes, and seconds.
e.g. `1h`    -> 1 hour
     `5m30s` -> 5 minutes, 30 seconds
     `600s`  -> 600 seconds
     `20`    -> 20 seconds

Default value: `10s`"
  )]
  fadeout_length: Option<Duration>,
  
  #[structopt(
    short = "o",
    long = "--output-dir",
    value_name = "dir",
    global = true,
    display_order = 2,
    help = "Path to the directory in which extracted files will be placed",
    long_help = 
r"Path to the directory in which extracted files will be placed.

The path string accepts a number of template parameters which will be filled in with game data from the bgminfo file. Template parameters are enclosed within double braces `{{}}`.

Supported values:
{{name_jp}}     - The game's full name, with original Japanese text
{{name_en}}     - The game's full name, with romanized Japanese text
{{name_short}}  - The portion of the game's name occuring after the `~`
{{game_number}} - The release number of the game within the Touhou series
                  Numbers < 10 are prefixed with a leading zero.

Default value: `{{name_jp}}`"
  )]
  output_dir: Option<String>,
  
  #[structopt(
    long = "--filename-format",
    value_name = "format",
    global = true,
    display_order = 3,
    help = "Format string specifying how filenames will be generated",
    long_help = 
r"Format string specifying how filenames will be generated.

The format string accepts a number of template parameters which will be filled in with track data from the bgminfo file. Template parameters are enclosed within double braces `{{}}`.

Supported values:
{{name_jp}}      - The track's full name, with original Japanese text
{{name_en}}      - The track's full name, with romanized Japanese text
{{track_number}} - The track number
                   Numbers < 10 are prefixed with a leading zero.

Default value: `{{track_number}} - {{name_jp}}`",
  )]
  filename_format: Option<String>,
  
}

/// Does anything show up here?
#[derive(StructOpt)]
enum Command {
  /// Generate tracks with a specified duration
  Length {
    #[structopt(
      value_name = "duration",
      parse(try_from_str = parse_duration::parse),
      help = "Length in seconds of each extracted track",
      long_help =
r"Length of each extracted track.

Value can be given as an integer value (in seconds), or as a duration with `h`, `m`, and `s` used to indicate hours, minutes, and seconds.
e.g. `10h`   -> 10 hours
     `5m30s` -> 5 minutes, 30 seconds
     `5m`    -> 5 minutes
     `300`   -> 300 seconds"
    )]
    length: Duration,
    
    #[structopt(
      parse(from_os_str),
      help = "Path to BGM file containing sound data offsets and game and track information",
      long_help = "Path to the BGM file containing sound data offsets and game and track information. This is the same BGM format as used by Touhou Music Room. For more information, see `https://en.touhouwiki.net/wiki/Game_Tools_and_Modifications#Bgm_files`.",
    )]
    bgminfo: PathBuf,
    
    #[structopt(
      value_name = "source-path",
      parse(from_os_str),
      help = "Path to source file to be extracted",
      long_help = "Path of the source to extract music from. For Embodiment of Scarlet Devil, this should be the path to the directory containing individual music files for the game. For all other mainline Touhou games, this will be the `thbgm.dat` file in the root game directory."
    )]
    source: PathBuf,
  },

  /// Generate tracks looped a specified number of times, with varying duration
  Looped {
    #[structopt(
      value_name = "loops",
      help = "Number of times to loop each extracted track",
      long_help = "Number of times to loop each extracted track."
    )]
    loop_count: usize,
    
    #[structopt(
      parse(from_os_str),
      help = "Path to BGM file containing sound data offsets and game and track information",
      long_help = "Path to the BGM file containing sound data offsets and game and track information. This is the same BGM format as used by Touhou Music Room. For more information, see `https://en.touhouwiki.net/wiki/Game_Tools_and_Modifications#Bgm_files`.",
    )]
    bgminfo: PathBuf,
    
    #[structopt(
      value_name = "source-path",
      parse(from_os_str),
      help = "Path of source to extract from",
      long_help = "Path of the source to extract music from. For Embodiment of Scarlet Devil, this should be the path to the directory containing individual music files for the game. For all other mainline Touhou games, this will be the `thbgm.dat` file in the root game directory."
    )]
    source: PathBuf,
    
    #[structopt(
      long = "--fade-before-loop",
      long_help = "By default, the extracted track is looped unfaded the full number of times specified, followed by a short fadeout of the start of what would be the next loop. When this flag is set, the track stops at the end of the final loop, with fadeout occuring up to that point."
    )]
    fade_before_loop: bool,
  },
}

fn main() -> Result<()> {
  let options = Options::from_args();
  
  let (output_mode, bgm_path, source_path) = match &options.mode {
    Command::Length {length, bgminfo, source} => {
      (
        OutputMode::FixedDuration(length.as_secs() as usize),
        bgminfo.to_path_buf(),
        source.to_path_buf(),
      )
    },
    Command::Looped {loop_count, bgminfo, source, fade_before_loop} => {
      let fade_mode = if *fade_before_loop {
        FadeMode::BeforeLoopPoint
      }
      else {
        FadeMode::AfterLoopPoint
      };

      (
        OutputMode::FixedLoops(*loop_count, fade_mode),
        bgminfo.to_path_buf(),
        source.to_path_buf(),
      )
    },
  };

  let bgm = if let Some(data) = BgmStore::get_from_token(&bgm_path.to_string_lossy()) {
    BgmInfo::try_from(data.as_ref())
  }
  else {
    BgmInfo::load_from_file(bgm_path)
  }?;

  // We have these parameters as optional in the Options struct and provide them with default values
  // here rather than setting a `default_value` field in structopt to prevent structopt from always
  // displaying them in the usage string for commands.
  let directory_format = options
                           .output_dir
                           .unwrap_or_else(|| "{{name_jp}}".to_owned());
  let filename_format = options
                          .filename_format
                          .unwrap_or_else(|| "{{track_number}} - {{name_jp}}".to_owned());
  let fadeout_duration = options
                           .fadeout_length
                           .map(|d| d.as_secs() as usize)
                           .unwrap_or(10);

  let opts = OutputOptions {
    directory_format,
    filename_format,
    output_mode: output_mode,
    fadeout_duration,
  };

  bgminfo::print_bgminfo(&bgm, options.track_number);
  //bgmstore::print_command_line_help();
  Ok(())
  //core::extract(&bgm, options.track_number, source_path, &opts)
}
