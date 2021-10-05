use super::{isample, Complex};

use std::error::Error;

pub fn compute_fft(input: &[isample], output: &mut [Complex<f32>]) -> Result<(), Box<dyn Error>>{
    let N: usize = input.len();
    let mut working_with = Vec::<Complex<f32>>::with_capacity(N);
    for x in input {
        working_with.push(Complex::new(*x as f32, 0.0));
    }

    let omega = -2f32 * std::f32::consts::PI / N as f32;
    let log = (N as f32).log2() as i32;
	let mut block_size = N;
	let mut num_blocks = 1;
    for i in 0..log as usize {   // Stage
		let half_block = block_size / 2;
        for j in 0..num_blocks { // Block
            for k in 0..half_block {   // Operation
				let ind_x = block_size * j + k;
				let ind_y = block_size * j + k + half_block;
                let src_x = working_with[ind_x];
                let src_y = working_with[ind_y];

                working_with[ind_x] = src_x + src_y;
                working_with[ind_y] = (src_x - src_y) * Complex::from_polar(&1.0, &(omega * (k * 2usize.pow(i as u32)) as f32));
            }
        }
		num_blocks *= 2;
		block_size /= 2;
    }

    let shift = std::mem::size_of::<usize>() * 8 - log as usize;
    for i in 0..N {
        output[i.reverse_bits() >> shift] = working_with[i];
    }

    Ok(())
}
