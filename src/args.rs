use std::error::Error;

macro_rules! throw {
	() => {
		print_help();
		return Err("Bad usage".into());
	};
}

fn print_help() {
	println!("
Usage: compressor ACTION SRC [DEST] [opts...]

Encoding:
	compressor encode SRC DEST [--ratio RATIO] [--storage-bits BITS]

RATIO is the compression ratio. Default 8, must be >= 1.
BITS is the number of bits to use in storing float values. Accepted values are 16 (default), 32.

Decoding:
	compressor decode SRC
");
}

pub enum Action {
	Encode,
	Decode,
}

pub struct Options {
	pub action: Action,
	pub src: String,
	pub dest: Option<String>,
	pub ratio: u32,
	pub storage: u8
}

pub fn parse_args() -> Result<Options, Box<dyn Error>> {
	let args = std::env::args().collect::<Vec<_>>();
	let mut opts = Options {
		action: Action::Encode,
		src: String::new(),
		dest: None,
		ratio: 8,
		storage: 1,
	};

	let mut offset = 0;
	if args.len() == 1 {
		throw!();
	} else if args[1] == "encode" {
		if args.len() < 4 {
			throw!();
		}

		opts.action = Action::Encode;
		opts.src = args[2].to_owned();
		opts.dest = Some(args[3].to_owned());
		offset = 4;
	} else if args[1] == "decode" {
		if args.len() < 3 {
			throw!();
		}

		opts.action = Action::Decode;
		opts.src = args[2].to_owned();
		opts.dest = None;
		offset = 3;
	} else {
		throw!();
	}

	while offset < args.len() {
		if args[offset] == "--ratio" {
			offset += 1;
			if offset >= args.len() {
				throw!();
			}

			let ratio = i32::from_str_radix(args[offset].as_str(), 10)?;
			if ratio < 1 {
				throw!();
			}

			opts.ratio = ratio as u32;
		} else if args[offset] == "--storage-bits" {
			offset += 1;
			if offset >= args.len() {
				throw!();
			}

			let storage_bits = match args[offset].as_str() {
				"16" => { 1 },
				"32" => { 2 },
				_ => {
					throw!();
				}
			};

			opts.storage = storage_bits;
		} else {
			throw!();
		}

		offset += 1;
	}

	Ok(opts)
}
