use std::error::Error;

macro_rules! throw {
	() => {
		return Err("Bad usage".into());
	};
}

fn print_help() {
	println!("
Usage: compressor ACTION SRC [DEST]

Encoding:
	compressor encode SRC DEST

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
	pub dest: Option<String>
}

pub fn parse_args() -> Result<Options, Box<dyn Error>> {
	let args = std::env::args().collect::<Vec<_>>();
	let mut opts = Options {
		action: Action::Encode,
		src: String::new(),
		dest: None,
	};

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
	} else if args[1] == "decode" {
		if args.len() < 3 {
			print_help();
			throw!();
		}

		opts.action = Action::Decode;
		opts.src = args[2].to_owned();
		opts.dest = None;
	} else {
		print_help();
		throw!();
	}

	Ok(opts)
}
