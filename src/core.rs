use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
  cmp::min,
  fs::File,
  io::{BufReader, BufWriter, Cursor, Read, Seek, Write},
  path::{PathBuf},
};

use crate::Result;
use crate::bgminfo::{BgmInfo, Track};
use crate::wavheader::WavHeader;

pub enum LoopedFadeMode {
  Before,
  After,
}

pub enum OutputMode {
  Loops(usize, LoopedFadeMode),
  Duration(usize),
}

pub struct OutputOptions {
  pub mode: OutputMode,
  pub fadeout_duration: u32,
}

pub fn process_track<W: Write>
(track: &Track, data: Vec<u8>, mut bw: BufWriter<W>, opts: &OutputOptions) -> Result<(usize, usize)> {
  let rel_loop = track.relative_loop_offset as usize;
  let rel_end  = track.relative_end_offset  as usize;

  let channels = 2;

  let intro_length  = rel_loop;
  let loop_length   = rel_end - rel_loop;
  let loop_duration = loop_length as u32 / (track.sample_rate * channels);

  // Fadeout duration is limited to the length of one full looped portion of the track. The number
  // of samples faded out is rounded down to the nearest 1000 for convenience of implementation.
  let fadeout_duration = min(opts.fadeout_duration, loop_duration);
  let fadeout_step_samples = (fadeout_duration * track.sample_rate * channels / 1000) as usize;
  let fadeout_samples  = fadeout_step_samples * 1000;
  let fadeout_bytes    = fadeout_samples * 2;

  let (loops, rel_fade_start) = match opts.mode {
    OutputMode::Loops(n, LoopedFadeMode::After)  => (n, rel_loop),
    OutputMode::Loops(n, LoopedFadeMode::Before) => (n - 1, rel_end - fadeout_bytes),
    OutputMode::Duration(duration) => {
      let total_samples = duration * track.sample_rate as usize * 2;
      let intro_samples = rel_loop / 2;
      let unfaded_loop_samples  = total_samples - intro_samples - fadeout_samples;
      let loop_duration_samples = (rel_end - rel_loop) / 2;

      let loops = unfaded_loop_samples / loop_duration_samples;
      let partial_loop_samples = unfaded_loop_samples % loop_duration_samples;
      let partial_loop_bytes   = partial_loop_samples * 2;

      let rel_fade_start = rel_loop + partial_loop_bytes * 2;

      (loops, rel_fade_start)
    },
  };

  let partial_loop_length = rel_fade_start - rel_loop;
  
  let length = intro_length + loops * loop_length + partial_loop_length + fadeout_bytes;

  let wave_header = WavHeader::new(length, track.sample_rate);
  wave_header.into_buf_writer(&mut bw)?;
  bw.write(&data[..rel_loop])?;   // Introduction
  for _ in 0..loops {             // Complete loops
    bw.write(&data[rel_loop..])?;
  }
  if rel_fade_start != rel_loop { // Partial unfaded loop
    bw.write(&data[rel_loop..rel_fade_start])?;
  }
  
  if fadeout_duration > 0
  {
    let mut fadeout_buffer = vec![0_i16; fadeout_samples];

    // Portion of the track that fades out is contained within a single loop
    if rel_fade_start + fadeout_bytes < rel_end {
      let mut c = Cursor::new(&data[rel_fade_start..(rel_fade_start + fadeout_bytes)]);
      c.read_i16_into::<LittleEndian>(&mut fadeout_buffer)?;
    }
    // Portion of the track that fades out is split across loops
    else {
      let pre_loop_bytes   = rel_end - rel_fade_start;
      let pre_loop_samples = pre_loop_bytes / 2;
      let post_loop_offset = rel_loop + fadeout_bytes - pre_loop_bytes;
      {
        let mut c = Cursor::new(&data[rel_fade_start..rel_end]);
        c.read_i16_into::<LittleEndian>(&mut fadeout_buffer[..pre_loop_samples])?;
      }
      {
        let mut c = Cursor::new(&data[rel_loop..post_loop_offset]);
        c.read_i16_into::<LittleEndian>(&mut fadeout_buffer[pre_loop_samples..])?;
      }
    }
    
    let mut fade_volume = 1.0;
    let mut start_offset = 0;

    for _ in 0..1000 {
      for index in start_offset..(start_offset + fadeout_step_samples) {
        let sample = (fadeout_buffer[index] as f32 * fade_volume) as i16;
        bw.write_i16::<LittleEndian>(sample)?;
      }

      fade_volume  = fade_volume - 0.001;
      start_offset = start_offset + fadeout_step_samples;
    }
  }

  Ok((0x28, length))
}

// pub fn extract_track<R: Read + Seek, W: Write>
// (track: &Track, source: &mut BufReader<R>, dest: BufWriter<W>, opts: &OutputOptions) -> Result<()> {
//   let mut data = vec![0; track.relative_end_offset as usize];
//   source.seek(std::io::SeekFrom::Start(track.start_offset))?;
//   source.read_exact(&mut data)?;

//   process_track(track, data, dest, opts)?;
//   Ok(())
// }

// pub fn e_a(bgm_info: BgmInfo, source: PathBuf, dest_dir: PathBuf, opts: &OutputOptions) -> Result<()> {
//   if !dest_dir.exists() {
//     std::fs::create_dir_all(dest_dir.clone())?;
//   }
//   else if !dest_dir.is_dir() {
//     return Err(format!("Destination path {:?} exists and is not a directory", dest_dir).into())
//   }

//   let mut infile = File::open(source)?;
//   let mut source = BufReader::new(infile);

//   for track in bgm_info.tracks {  
//     // infile.seek(std::io::SeekFrom::Start(track.start_offset))?;
//     // let mut data = vec![0; track.relative_end_offset as usize];
//     // infile.read_exact(&mut data)?;

//     let dest_path = dest_dir.join(format!("{:02}.wav", track.track_number));
//     let file = File::create(dest_path)?;
//     let bw = BufWriter::new(file);

//     extract_track(&track, &mut source, bw, opts)?;
//   }

//   Ok(())
// }

pub fn extract_all(bgm_info: BgmInfo, source: PathBuf, dest_dir: PathBuf, opts: &OutputOptions) -> Result<()> {
  if !dest_dir.exists() {
    std::fs::create_dir_all(dest_dir.clone())?;
  }
  else if !dest_dir.is_dir() {
    return Err(format!("Destination path {:?} exists and is not a directory", dest_dir).into())
  }

  let mut infile = File::open(source)?;
  let mut source = BufReader::new(infile);

  for track in bgm_info.tracks {  
    source.seek(std::io::SeekFrom::Start(track.start_offset))?;
    let mut data = vec![0; track.relative_end_offset as usize];
    source.read_exact(&mut data)?;

    let dest_path = dest_dir.join(format!("{:02}.wav", track.track_number));
    let file = File::create(dest_path)?;
    let bw = BufWriter::new(file);

    process_track(&track, data, bw, opts)?;
  }

  Ok(())
}