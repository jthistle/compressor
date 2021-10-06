use super::{isample, frequency_to_discrete, Complex, Chunk};

use std::convert::TryInto;
use std::fs;
use std::error::Error;
use std::io::{Write};

extern crate half;
use half::{bf16};

mod fft;

mod riff;
use riff::RiffChunk;

pub struct EncodeOptions {
	pub chunk_size: usize,
	pub out_size: usize,
	pub storage_type: u8,
}


/// Analyzes a chunk for frequency spectrum, discards useless frequencies, and appends `out_size` results to
/// the `out` array.
fn encode_chunk(chunk: &[isample], out: &mut Vec<Chunk>, out_size: usize, fs: f32) -> Result<(), Box<dyn Error>> {
	let in_size = chunk.len();

	let mut fft_output = std::iter::repeat(Complex::new(0.0, 0.0)).take(in_size).collect::<Vec<_>>();
	fft::compute_fft(&chunk, &mut fft_output[..])?;

	let mut out_tmp = std::iter::repeat((0u16, Complex::new(0.0, 0.0))).take(out_size).collect::<Vec<_>>();
	// let skip = (frequency_to_discrete(20.0, fs, in_size) - 1).max(0) as usize;
	let skip = 0;
	let limit = frequency_to_discrete(20000.0, fs, in_size) as usize;
	for (ind, val) in fft_output.into_iter().skip(skip).take(limit - skip).enumerate() {
		if val.norm() < out_tmp[out_size - 1].1.norm() { continue; }
		for i in 0..out_size {
			if val.norm() > out_tmp[i].1.norm() {
				out_tmp.pop();
				out_tmp.insert(i, (ind as u16, val));
				break;
			}
		}
	}

	out.append(&mut out_tmp);

	Ok(())
}

#[inline(always)]
fn write_chunk_f32(out_file: &mut fs::File, chunk: &Chunk) -> Result<(), Box<dyn Error>> {
	out_file.write_all(&(chunk.1.re).to_le_bytes())?;
	out_file.write_all(&(chunk.1.im).to_le_bytes())?;
	Ok(())
}

#[inline(always)]
fn write_chunk_f16(out_file: &mut fs::File, chunk: &Chunk) -> Result<(), Box<dyn Error>> {
	out_file.write_all(&(bf16::from_f32(chunk.1.re)).to_le_bytes())?;
	out_file.write_all(&(bf16::from_f32(chunk.1.im)).to_le_bytes())?;
	Ok(())
}

fn write_chunks(chunks: &[Vec<Chunk>], out_size: usize, chunk_size: usize, fs: f32, storage_type: u8, dest: &str) -> Result<(), Box<dyn Error>> {
	let mut out_file = fs::File::create(dest)?;
	let num_channels = chunks.len();

	out_file.write_all("XPRS".as_bytes())?;
	out_file.write_all(&(chunk_size as i32).to_le_bytes())?;
	out_file.write_all(&(chunks[0].len() as i32).to_le_bytes())?;
	out_file.write_all(&(out_size as i32).to_le_bytes())?;
	out_file.write_all(&(fs as i32).to_le_bytes())?;
	out_file.write_all(&(chunks.len() as i32).to_le_bytes())?;
	out_file.write_all(&(storage_type as u16).to_le_bytes())?;
	out_file.write_all("DATA".as_bytes())?;

	let write_chunk_data = match storage_type {
		1 => { write_chunk_f16 },
		2 => { write_chunk_f32 },
		_ => { return Err("Invalid storage type".try_into()?) }
	};

	// Write interleaved
	for i in 0..chunks[0].len() {
		for j in 0..num_channels {
			let chunk = chunks[j][i];
			out_file.write_all(&(chunk.0 as u16).to_le_bytes())?;
			write_chunk_data(&mut out_file, &chunk)?;
		}
	}

	Ok(())
}

pub fn encode(filename: &str, destination: &str, opts: EncodeOptions) -> Result<(), Box<dyn Error>> {
    let raw = fs::read(filename)?;

    let (fs, channels) =
    {
        let root = RiffChunk::new(&raw[..])?;

		let num_channels = root.child("fmt ").unwrap().data.as_ref().unwrap()[2] as usize;
		let sample_rate = i32::from_le_bytes(root.child("fmt ").unwrap().data.as_ref().unwrap()[4..8].try_into()?);
        let data = root.child("data").unwrap().data.as_ref().unwrap();

        // Convert data from Vec<u8> to Vec<isample>
        // We assume 16-bit samples, LE
        let capacity = data.len() / 4;
        let mut channels = Vec::<Vec<isample>>::new();
		for _ in 0..num_channels {
			channels.push(Vec::with_capacity(capacity));
		}
        for sample in 0..capacity {
           for channel in 0..num_channels {
               channels[channel].push(isample::from_le_bytes(data[sample*4+channel*2..sample*4+channel*2+2].try_into()?));
           }
        }

        (sample_rate as f32, channels)
    };

	let mut out = Vec::new();
	let chunk_size = opts.chunk_size;
	let out_size = opts.out_size;
	let num_chunks = channels[0].len() / chunk_size;
	let num_channels = channels.len();

	let part = num_channels * num_chunks / 100;
	for j in 0..num_channels {
		out.push(Vec::new());
		for i in 0..num_chunks {
			if (i + num_chunks * j) % part == 0 {
				println!("{}%", (i + num_chunks * j) / part );
			}
			encode_chunk(&channels[j][i * chunk_size..(i + 1) * chunk_size], &mut out[j], out_size, fs)?;
		}
	}

	println!("writing...");
	write_chunks(&out[..], out_size, chunk_size, fs, opts.storage_type, destination)?;

    Ok(())
}
