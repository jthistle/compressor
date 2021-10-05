use super::{isample};

use std::error::Error;
use std::fs;
use std::convert::TryInto;

extern crate alsa;
use alsa::pcm::{Frames};
use alsa::{ValueOr, Direction};

mod fft;


fn play(samples: &Vec<isample>, fs: u32) -> Result<(), Box<dyn Error>> {
    let pcm = alsa::pcm::PCM::new("default", Direction::Playback, false)?;
	let period_size = 512;

    // Setup pcm params
    {
        let params = alsa::pcm::HwParams::any(&pcm)?;

        params.set_channels(1).unwrap();
        params.set_rate(fs, ValueOr::Nearest)?;
        params.set_format(alsa::pcm::Format::s16())?;
        params.set_access(alsa::pcm::Access::RWInterleaved)?;
        params.set_period_size(period_size as Frames, ValueOr::Nearest)?;
        params.set_buffer_size(8 * 512)?;

        pcm.hw_params(&params)?;
    }

	let io = pcm.io_i16()?;

	for i in 0..samples.len() / period_size {
		io.writei(&samples[i * period_size..(i + 1) * period_size])?;
	}

	pcm.drain()?;

    Ok(())
}


fn generate_from_chunk(chunks: &[(u16, f32)], out: &mut Vec<isample>, chunk_size: usize) -> Result<(), Box<dyn Error>> {
	let mut transform = std::iter::repeat(0f32).take(chunk_size).collect::<Vec<_>>();

	for chunk in chunks {
		transform[chunk.0 as usize] = chunk.1;
	}

	// println!("{:?}", transform);

	out.append(&mut std::iter::repeat(0 as isample).take(chunk_size as usize).collect::<Vec<_>>());
	let start_ind = out.len() - chunk_size;
	fft::compute_inverse_fft(&transform, &mut out[start_ind..])?;

	Ok(())
}


pub fn decode(filename: &str) -> Result<(), Box<dyn Error>> {
    let raw = fs::read(filename)?;

	let chunk_size = i32::from_le_bytes(raw[4..8].try_into()?) as usize;
	let chunk_count = i32::from_le_bytes(raw[8..12].try_into()?) as usize;
	let out_size = i32::from_le_bytes(raw[12..16].try_into()?) as usize;
	let fs = i32::from_le_bytes(raw[16..20].try_into()?);

	let chunks = {
		let mut chunks = Vec::<(u16, f32)>::with_capacity(chunk_size);
		for i in 0..chunk_count {
			let start = 24 + i * 6;
			let freq = u16::from_le_bytes(raw[start..start+2].try_into()?);
			let mag = f32::from_le_bytes(raw[start+2..start+6].try_into()?);
			chunks.push((freq, mag));
		}
		chunks
	};

	let mut wave = Vec::<isample>::new();
	println!("decode");
	for i in (0..chunk_count).step_by(out_size) {
		generate_from_chunk(&chunks[i..i + out_size], &mut wave, chunk_size)?;
	}

	println!("play");
	play(&wave, fs as u32)?;

	Ok(())
}
