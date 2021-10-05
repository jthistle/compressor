use super::{isample, frequency_to_discrete, Complex, Chunk};

use std::convert::TryInto;
use std::fs;
use std::error::Error;
use std::io::{Write};

pub mod fft;

mod riff;
use riff::RiffChunk;

/// Analyzes a chunk for frequency spectrum, discards useless frequencies, and appends `out_size` results to
/// the `out` array.
fn encode_chunk(chunk: &[isample], out: &mut Vec<Chunk>, out_size: usize, fs: f32) -> Result<(), Box<dyn Error>> {
	let in_size = chunk.len();

	let mut fft_output = std::iter::repeat(Complex::new(0.0, 0.0)).take(in_size).collect::<Vec<_>>();
	fft::compute_fft(&chunk[..], &mut fft_output[..])?;

	let mut out_tmp = std::iter::repeat((0u16, Complex::new(0.0, 0.0))).take(out_size).collect::<Vec<_>>();
	let limit = frequency_to_discrete(20000.0, fs, in_size) as usize;
	for (ind, val) in fft_output.into_iter().take(limit).enumerate() {
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

fn write_chunks(chunks: &Vec<Chunk>, out_size: usize, chunk_size: usize, fs: f32, dest: &str) -> Result<(), Box<dyn Error>> {
	let mut out_file = fs::File::create(dest)?;

	out_file.write("XPRS".as_bytes())?;
	out_file.write(&(chunk_size as i32).to_le_bytes())?;
	out_file.write(&(chunks.len() as i32).to_le_bytes())?;
	out_file.write(&(out_size as i32).to_le_bytes())?;
	out_file.write(&(fs as i32).to_le_bytes())?;
	out_file.write("DATA".as_bytes())?;

	// Normalize
	/*
	let mut max = 0f32;
	for i in (0..chunks.len()).step_by(chunk_size) {
		if chunks[i].1.norm() > max  {
			max = chunks[i].1;
		}
	}
	*/

	for chunk in chunks {
		out_file.write(&(chunk.0 as u16).to_le_bytes())?;
		out_file.write(&(chunk.1.re as f32).to_le_bytes())?;
		out_file.write(&(chunk.1.im as f32).to_le_bytes())?;
	}

	Ok(())
}

pub fn encode(filename: &str, destination: &str) -> Result<(), Box<dyn Error>> {
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
           for channel in 0..1 {
               channels[channel].push(isample::from_le_bytes(data[sample*4+channel*2..sample*4+channel*2+2].try_into()?));
           }
        }

        (sample_rate as f32, channels)
    };

	let mut out = Vec::new();
	let chunk_size = 256 * 4;
	let out_size = 64;
	let num_chunks = channels[0].len() / chunk_size;

	let part = num_chunks / 100;
	for i in 0..num_chunks {
		if i % part == 0 {
			println!("{}%", i / part);
		}
		encode_chunk(&channels[0][i * chunk_size..(i + 1) * chunk_size], &mut out, out_size, fs)?;
	}

	/*
	for i in 0..num_chunks {
		println!("\nChunk {}", i);
		for j in 0..out_size {
			let chunk = out[i * out_size + j];
			println!("{}: {} mag, {} Hz", j, chunk.0, discrete_to_frequency(chunk.1, fs, chunk_size));
		}
	}
	*/

	write_chunks(&out, out_size, chunk_size, fs, destination)?;

    Ok(())
}
