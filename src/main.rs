// use nom::{
//     IResult,
//     bytes::complete::take,
//     multi::many0,
//     number::complete::be_u32,
// };

// #[derive(Debug)]
// struct BgmRecord {
//     wav_name: Vec<u8>,
//     start_offset: u32,
//     intro_length: u32,
//     total_length: u32,
//     riff_header: Vec<u8>,
// }

// fn bgm_record(input: &[u8]) -> IResult<&[u8], BgmRecord> {
//     let (input, wav_name)     = take(16usize)(input)?;
//     let (input, start_offset) = be_u32(input)?;
//     let (input, _)            = take(4usize)(input)?;
//     let (input, intro_length) = be_u32(input)?;
//     let (input, total_length) = be_u32(input)?;
//     let (input, riff_header)  = take(18usize)(input)?;

//     Ok((input, BgmRecord {wav_name: wav_name.to_owned(), start_offset, intro_length, total_length, riff_header: riff_header.to_owned()}))
// }

// fn bgm_records(input: &[u8]) -> IResult<&[u8], Vec<BgmRecord>> {
//     many0(bgm_record)(input)
// }

use bincode;
use serde::{Serialize, Deserialize};
use std::{
  error::Error,
  io::{BufWriter, Write},
};

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
  path::Path,
};

fn main() -> Result<()> {
  let start_offset: u64 = 0x18BC4930;
  let rel_loop: usize = 0x00055500;
  let rel_end: usize = 0x01C28000;
  let loops = 1;

  let inpath = Path::new("/mnt/e/Torrents/Touhou/7/Perfect Cherry Blossom/Thbgm.dat");
  let mut infile = File::open(inpath)?;
  infile.seek(std::io::SeekFrom::Start(start_offset))?;
  let mut data = vec![0; rel_end];
  infile.read_exact(&mut data)?;
  let rate: u32 = 441000;

  let path = Path::new("Necrofantasia.wav");
  let file = File::create(&path)?;
  let mut bw = BufWriter::new(file);

  let wave = WavFile::new(data.len(), rate);
  wave.into_buf_writer(&mut bw)?;
  bw.write(data.as_slice())?;

  Ok(())
}
