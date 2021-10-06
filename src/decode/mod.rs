use super::{isample, Chunk, Complex};

use std::error::Error;
use std::fs;
use std::convert::{TryInto};

extern crate half;
use half::bf16;

extern crate alsa;
use alsa::pcm::{Frames};
use alsa::{ValueOr, Direction};

mod fft;


fn play(samples: &[isample], channels: usize, fs: u32) -> Result<(), Box<dyn Error>> {
    let pcm = alsa::pcm::PCM::new("default", Direction::Playback, false)?;
	let period_size = 512usize;

    // Setup pcm params
    {
        let params = alsa::pcm::HwParams::any(&pcm)?;

        params.set_channels(channels as u32).unwrap();
        params.set_rate(fs, ValueOr::Nearest)?;
        params.set_format(alsa::pcm::Format::s16())?;
        params.set_access(alsa::pcm::Access::RWInterleaved)?;
        params.set_period_size(period_size as Frames, ValueOr::Nearest)?;
        params.set_buffer_size(8 * period_size as Frames)?;

        pcm.hw_params(&params)?;
    }

	let io = pcm.io_i16()?;

	for i in 0..samples.len() / period_size {
		match io.writei(&samples[i * period_size..(i + 1) * period_size]) {
			Ok(_) => {},
			Err(e) => {
				pcm.try_recover(e, false)?;
			}
		}
	}

	pcm.drain()?;

    Ok(())
}

fn generate_from_chunk(chunks: &[Chunk], out: &mut Vec<isample>, chunk_size: usize) -> Result<(), Box<dyn Error>> {
	let mut transform = std::iter::repeat(Complex::new(0.0, 0.0)).take(chunk_size).collect::<Vec<_>>();

	for chunk in chunks {
		transform[chunk.0 as usize] = chunk.1;
	}

	out.append(&mut std::iter::repeat(0).take(chunk_size as usize).collect::<Vec<_>>());
	let start_ind = out.len() - chunk_size;
	fft::compute_inverse_fft(&transform, &mut out[start_ind..])?;

	Ok(())
}

fn interleave(wave: &[Vec<isample>]) -> Vec<isample> {
	let num_chans = wave.len();
	let chan_size = wave[0].len();
	let mut out = Vec::with_capacity(num_chans * chan_size);

	for i in 0..chan_size {
		for j in 0..num_chans {
			out.push(wave[j][i]);
		}
	}

	out
}

#[inline(always)]
fn read_number_f32(raw: &[u8]) -> Result<Complex<f32>, Box<dyn Error>> {
	let re = f32::from_le_bytes(raw[0..4].try_into()?);
	let im = f32::from_le_bytes(raw[4..8].try_into()?);
	Ok(Complex::new(re, im))
}

#[inline(always)]
fn read_number_f16(raw: &[u8]) -> Result<Complex<f32>, Box<dyn Error>> {
	let re = bf16::from_le_bytes(raw[0..2].try_into()?);
	let im = bf16::from_le_bytes(raw[2..4].try_into()?);
	Ok(Complex::new(re.to_f32(), im.to_f32()))
}

pub fn decode(filename: &str) -> Result<(), Box<dyn Error>> {
    let raw = fs::read(filename)?;

	let chunk_size = i32::from_le_bytes(raw[4..8].try_into()?) as usize;
	let chunk_count = i32::from_le_bytes(raw[8..12].try_into()?) as usize;
	let out_size = i32::from_le_bytes(raw[12..16].try_into()?) as usize;
	let fs = i32::from_le_bytes(raw[16..20].try_into()?);
	let num_channels = i32::from_le_bytes(raw[20..24].try_into()?) as usize;
	let storage_type = u16::from_le_bytes(raw[24..26].try_into()?) as u8;

	let chunk_bytes = match storage_type {
		1 => { 6 },
		2 => { 10 },
		_ => { return Err("Invalid storage type".try_into()?) }
	};

	let read_number = match storage_type {
		1 => { read_number_f16 },
		2 => { read_number_f32 },
		_ => { return Err("Invalid storage type".try_into()?) }
	};

	let chunks = {
		let mut chunks = Vec::new();
		for j in 0..num_channels {
			let mut channel = Vec::<Chunk>::with_capacity(chunk_size);
			for i in 0..chunk_count {
				let start = 30 + (i * num_channels + j) * chunk_bytes;
				let freq = u16::from_le_bytes(raw[start..start+2].try_into()?);
				channel.push((freq, read_number(&raw[start+2..start+chunk_bytes])?));
			}
			chunks.push(channel);
		}
		chunks
	};

	let mut wave = Vec::new();
	println!("decode");
	for j in 0..num_channels {
		let mut chan = Vec::new();
		for i in (0..chunk_count).step_by(out_size) {
			generate_from_chunk(&chunks[j][i..i + out_size], &mut chan, chunk_size)?;
		}
		wave.push(chan);
	}

	println!("play");
	let samples = interleave(&wave[..]);
	play(&samples[..], num_channels, fs as u32)?;
	// play(&wave[0], 1, fs as u32)?;

	Ok(())
}
