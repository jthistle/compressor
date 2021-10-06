use std::error::Error;

mod encode;
use encode::{EncodeOptions};

mod decode;

mod args;
use args::{parse_args, Action};

extern crate num_complex;
pub use num_complex::Complex;

pub type Chunk = (u16, Complex<f32>);
pub type isample = i16;


pub fn discrete_to_frequency(k: i32, fs: f32, n: usize) -> f32 {
	fs * k as f32 / n as f32
}

pub fn frequency_to_discrete(f: f32, fs: f32, n: usize) -> i32 {
	(f * n as f32 / fs) as i32
}


fn main() -> Result<(), Box<dyn Error>> {
	let opts = parse_args()?;

	match opts.action {
		Action::Encode => {
			let encode_opts = EncodeOptions {
				chunk_size: 1024,
				out_size: 128,
			};
			encode::encode(
				opts.src.as_str(),
				opts.dest.unwrap().as_str(),
				encode_opts
			)?;
		},
		Action::Decode => {
			decode::decode(opts.src.as_str())?;
		}
	}

	Ok(())
}
