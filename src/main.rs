use std::error::Error;

mod encode;
use encode::fft;

mod decode;


pub type isample = i16;

pub fn discrete_to_frequency(k: i32, fs: f32, n: usize) -> f32 {
	fs * k as f32 / n as f32
}

pub fn frequency_to_discrete(f: f32, fs: f32, n: usize) -> i32 {
	(f * n as f32 / fs) as i32
}


fn fft_test() -> Result<(), Box<dyn Error>> {
	let mut out = std::iter::repeat(0f32).take(16).collect::<Vec<_>>();
	fft::compute_fft(&vec![1,2,3,4,5,6,7,8,8,7,6,5,4,3,2,1], &mut out)?;
	println!("{:?}", out);

	let mut out = std::iter::repeat(0f32).take(16).collect::<Vec<_>>();
	fft::compute_fft_2(&vec![1,2,3,4,5,6,7,8,8,7,6,5,4,3,2,1], &mut out)?;
	println!("{:?}", out);

	Ok(())
}


fn main() -> Result<(), Box<dyn Error>> {
	// fft_test()
	encode::encode("anile.wav", "anile.xprs")
	// decode::decode("anile.xprs")
}
