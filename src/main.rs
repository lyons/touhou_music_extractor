use bincode;
use serde::{Serialize, Deserialize};
use std::{
  error::Error,
  io::{BufWriter, Write},
};

mod bgminfo;

pub(crate) type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
struct RiffHeader {
  chunkID: [u8; 4],   // 'RIFF'
  chunkSize: u32,     // 32 + WavData chunk size
  format: [u8; 4],    // 'WAVE'
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
struct WavFormat {
  chunkID: [u8; 4],   // 'fmt '
  chunkSize: u32,     // 16
  audioFormat: u16,   // 1 (PCM)
  numChannels: u16,   // 2 (Stereo)
  sampleRate: u32,    // 44100, except for spirit world themes in Ten Desires which are 22050
  byteRate: u32,      // sampleRate * 2 * 16/8
  blockAlign: u16,    // 2 * 16/8
  bitsPerSample: u16, // 16
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct DataHeader {
  chunkId: [u8; 4],   // 'data'
  chunkSize: u32,     // data.length()
}

#[derive(Debug, Serialize, Deserialize)]
struct WavFile {
  riff_header: RiffHeader,
  wave_format: WavFormat,
  data_header: DataHeader,
}

impl WavFile {
  fn new(length: usize, sample_rate: u32) -> WavFile {
    let byte_rate = sample_rate * 4;
    let data_size = length as u32;

    WavFile {
      riff_header: RiffHeader {chunkID: [0x52, 0x49, 0x46, 0x46], 
                               chunkSize: 36 + data_size, 
                               format: [0x57, 0x41, 0x56, 0x45]},
      wave_format: WavFormat {chunkID: [0x66, 0x6D, 0x74, 0x20],
                              chunkSize: 16,
                              audioFormat: 1,
                              numChannels: 2,
                              sampleRate: sample_rate,
                              byteRate: byte_rate,
                              blockAlign: 4,
                              bitsPerSample: 16},
      data_header: DataHeader {chunkId: [0x64, 0x61, 0x74, 0x61],
                               chunkSize: data_size},
    }
  }

  fn into_buf_writer<W: Write>(&self, writer: &mut BufWriter<W>) -> Result<()> {
    bincode::serialize_into(writer, self).map_err(|e| e.into())
  }
}

use std::{
  fs::File,
  io::{Read, Seek},
  path::{Path, PathBuf},
};
use structopt::StructOpt;
use crate::bgminfo::BgmInfo;

fn extract_demo() -> Result<()> {
  let start_offset: u64 = 0x18BC4930;
  let rel_loop: usize = 0x00055500;
  let rel_end: usize = 0x01C28000;
  let loops = 3;

  let inpath = Path::new("/mnt/e/Torrents/Touhou/7/Perfect Cherry Blossom/Thbgm.dat");
  let mut infile = File::open(inpath)?;
  infile.seek(std::io::SeekFrom::Start(start_offset))?;
  let mut data = vec![0; rel_end];
  infile.read_exact(&mut data)?;
  let rate: u32 = 44100;

  let path = Path::new("Necrofantasia.wav");
  let file = File::create(&path)?;
  let mut bw = BufWriter::new(file);

  let intro_length = rel_loop;
  let loop_length = rel_end - rel_loop;
  let length = intro_length + loops * loop_length;
  let wave = WavFile::new(length, rate);
  wave.into_buf_writer(&mut bw)?;
  bw.write(&data[..rel_loop])?;
  for _ in 0..loops {
    bw.write(&data[rel_loop..])?;
  }

  Ok(())
}

fn extract_all(bgm_info: BgmInfo, source: PathBuf, dest_dir: PathBuf, loops: u32) -> Result<()> {
  if !dest_dir.exists() {
    std::fs::create_dir_all(dest_dir.clone())?;
  }
  else if !dest_dir.is_dir() {
    return Err(format!("Destination path {:?} exists and is not a directory", dest_dir).into())
  }

  let mut infile = File::open(source)?;

  for track in bgm_info.tracks {
    let start_offset: u64 = track.position[0];
    let rel_loop: usize = track.position[1] as usize;
    let rel_end: usize = track.position[2] as usize;
  
    infile.seek(std::io::SeekFrom::Start(start_offset))?;
    let mut data = vec![0; rel_end];
    infile.read_exact(&mut data)?;

    let dest_path = dest_dir.join(format!("{:02}.wav", track.track_number));
    let file = File::create(dest_path)?;
    let mut bw = BufWriter::new(file);

    let intro_length = rel_loop;
    let loop_length = rel_end - rel_loop;
    let length = intro_length + (loops as usize) * loop_length;
    let wave = WavFile::new(length, track.frequency);
    wave.into_buf_writer(&mut bw)?;
    bw.write(&data[..rel_loop])?;
    for _ in 0..loops {
      bw.write(&data[rel_loop..])?;
    }
  }

  Ok(())
}

#[derive(StructOpt)]
struct Options {
  #[structopt(parse(from_os_str))]
  bgminfo: PathBuf,
  #[structopt(parse(from_os_str), default_value = "thbgm.dat")]
  source: PathBuf,
  #[structopt(parse(from_os_str), default_value = "output/")]
  dest: PathBuf,
}

fn main() -> Result<()> {
  let options = Options::from_args();

  let data = std::fs::read_to_string(options.bgminfo)?;
  let bgm: bgminfo::BgmInfo = toml::from_str(&data)?;

  if !bgm.game.packmethod == 2 {
    panic!("Unsupported pack method: {}", bgm.game.packmethod);
  }
  
  //for track in bgm.tracks {
  //  println!("{:02} - {}", track.track_number, track.name_jp);
  //}
  //let ip = Path::new("/mnt/e/Torrents/Touhou/10/Mountain of Faith/thbgm.dat");
  //let op = Path::new("MoF/");
  //extract_all(bgm, ip, op, 1)?;
  extract_all(bgm, options.source, options.dest, 1)?;

  Ok(())
}
