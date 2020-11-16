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
struct WavHeader {
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
  blockAlign: u32,    // 2 * 16/8
  bitsPerSample: u32, // 16
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct DataHeader {
  chunkId: [u8; 4],   // 'data'
  chunkSize: u32,     // data.length()
}

#[derive(Debug, Serialize, Deserialize)]
struct WavFile {
  riff_header: WavHeader,
  wave_format: WavFormat,
  data_header: DataHeader,
  data: Vec<u8>,
}

impl WavFile {
  fn new(data: Vec<u8>, sample_rate: u32) -> WavFile {
    let byte_rate = sample_rate * 4;
    let data_size = data.len() as u32;

    WavFile {
      riff_header: WavHeader {chunkID: [0x52, 0x49, 0x46, 0x46], 
                              chunkSize: 32 + data_size, 
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
      data: data,
    }
  }

  fn into_buf_writer<W: Write>(&self, writer: BufWriter<W>) -> Result<()> {
    bincode::serialize_into(writer, self).map_err(|e| e.into())
  }
}

fn main() -> Result<()> {
  let data: Vec<u8> = vec![0, 0, 0, 0];
  let rate: u32 = 441000;

  let path = std::path::Path::new("test.wav");
  let file = std::fs::File::create(&path)?;
  let bw = BufWriter::new(file);

  let wave = WavFile::new(data, rate);
  wave.into_buf_writer(bw)?;

  Ok(())
}
