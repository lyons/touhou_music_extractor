use bincode;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Serialize, Deserialize};
use std::{
  error::Error,
  io::{BufWriter, Cursor, Write},
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
  path::{PathBuf},
};
use structopt::StructOpt;
use crate::bgminfo::BgmInfo;

fn extract_all(bgm_info: BgmInfo, source: PathBuf, dest_dir: PathBuf, loops: u32) -> Result<()> {
  if !dest_dir.exists() {
    std::fs::create_dir_all(dest_dir.clone())?;
  }
  else if !dest_dir.is_dir() {
    return Err(format!("Destination path {:?} exists and is not a directory", dest_dir).into())
  }

  let mut infile = File::open(source)?;

  for track in bgm_info.tracks {
    // Recalculate offsets in terms of numbers of 16-bit samples rather than bytes
    // Or not
    let rel_loop = track.relative_loop_offset as usize;
    let rel_end = track.relative_end_offset as usize;
  
    infile.seek(std::io::SeekFrom::Start(track.start_offset))?;
    let mut data = vec![0; rel_end];
    infile.read_exact(&mut data)?;
    //infile.read_u16_into::<LittleEndian>(&mut data)?;

    let dest_path = dest_dir.join(format!("{:02}.wav", track.track_number));
    let file = File::create(dest_path)?;
    let mut bw = BufWriter::new(file);

    // fadeout
    // fadeout_samples = duration (seconds) * sample rate -> round down to nearest thousand
    // allocate fadeout buffer

    let fadeout_duration = 10;
    let fadeout_block = ((fadeout_duration * track.sample_rate / 1000) * 2) as usize;
    let fadeout_samples = fadeout_block * 1000;

    // fadeout during last loop
    // write [rel_loop..(rel_end - fadeout_samples)] from data to bufwriter
    // copy [(rel_end - fadeout_samples)..] into fadeout buffer

    // fadeout after last loop
    // write last loop of data to bufwriter
    // copy [..fadeout_samples] to fadeout buffer

    // start fadelevel at 1.000
    // every fadeout_samples / 1000, decrease fadelevel by 0.001
    // write sample * fadelevel to bufwriter

    let intro_length = rel_loop;
    let loop_length = rel_end - rel_loop;
    let length = intro_length + (loops as usize) * loop_length;
    let wave = WavFile::new(length, track.sample_rate);
    wave.into_buf_writer(&mut bw)?;
    bw.write(&data[..rel_loop])?;
    for _ in 0..loops {
      bw.write(&data[rel_loop..])?;
    }
    
    {
      let mut fadeout_buffer = vec![0_u16; fadeout_samples];
      let mut c = Cursor::new(&data[..fadeout_samples]);
      c.read_u16_into::<LittleEndian>(&mut fadeout_buffer)?;
      
      let mut fade_volume = 1.0;
      let mut start_offset = 0;

      for _ in 0..1000 {
        for index in start_offset..(start_offset + fadeout_block) {
          fadeout_buffer[index] = ((fadeout_buffer[index] as f32) * fade_volume) as u16;
        }

        fade_volume = fade_volume - 0.001;
        start_offset = start_offset + fadeout_block;
      }

      for sample in fadeout_buffer {
        bw.write_u16::<LittleEndian>(sample)?;
      }
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
  
  let bgm = bgminfo::load(options.bgminfo)?;

  match bgm.game.pack_method {
    bgminfo::PackMethod::Two(_, _, _) => {
      extract_all(bgm, options.source, options.dest, 1)
    },
    _ => {
      Err(format!("Unsupported pack method: {:?}", bgm.game.pack_method).into())
    },
  }
}
