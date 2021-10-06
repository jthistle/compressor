use std::error::Error;

macro_rules! throw {
	() => {
		return Err("Bad usage".into());
	};
}

fn print_help() {
	println!("
Usage: compressor ACTION SRC [DEST] [opts...]

Encoding:
	compressor encode SRC DEST [--ratio RATIO]

RATIO is the compression ratio. Default 8, must be >= 1.

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
}

pub fn parse_args() -> Result<Options, Box<dyn Error>> {
	let args = std::env::args().collect::<Vec<_>>();
	let mut opts = Options {
		action: Action::Encode,
		src: String::new(),
		dest: None,
		ratio: 8,
	};

	let mut offset = 0;
	if args.len() == 1 {
		print_help();
		throw!();
	} else if args[1] == "encode" {
		if args.len() < 4 {
			print_help();
			throw!();
		}

		opts.action = Action::Encode;
		opts.src = args[2].to_owned();
		opts.dest = Some(args[3].to_owned());
		offset = 4;
	} else if args[1] == "decode" {
		if args.len() < 3 {
			print_help();
			throw!();
		}

		opts.action = Action::Decode;
		opts.src = args[2].to_owned();
		opts.dest = None;
		offset = 3;
	} else {
		print_help();
		throw!();
	}

	while offset < args.len() {
		if args[offset] == "--ratio" {
			offset += 1;
			if offset >= args.len() {
				print_help();
				throw!();
			}

			let ratio = i32::from_str_radix(args[offset].as_str(), 10)?;
			if ratio < 1 {
				print_help();
				throw!();
			}

			opts.ratio = ratio as u32;
		} else {
			print_help();
			throw!();
		}

		offset += 1;
	}

	Ok(opts)
}
