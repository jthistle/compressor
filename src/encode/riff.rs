use std::borrow::Borrow;
use std::error::Error;
use std::convert::{TryInto, From};

#[derive(Debug)]
pub struct RiffChunk {
	tag: String,
	/// The 'length' of a chunk is always 8 bytes less than the actual length of chunk
	length: i32,
	pub data: Option<Vec<u8>>,
	children: Option<Vec<RiffChunk>>,
	form_type: Option<String>,
}

impl RiffChunk {
	pub fn new(raw: &[u8]) -> Result<RiffChunk, Box<dyn Error>> {
		let tag = String::from_utf8(raw[0..4].to_vec())?;
		let true_length = i32::from_le_bytes(raw[4..8].try_into()?);
		let length = if true_length % 2 == 1 { true_length + 1 } else { true_length };

		let (data, children, form_type) = {
			if tag == "RIFF" || tag == "LIST" {
				let form_type = String::from_utf8(raw[8..12].to_vec())?;
				let mut children = Vec::<RiffChunk>::new();

				let mut offset = 12usize;
				while offset < 8 + length as usize {
					let new_child = RiffChunk::new(&raw[offset..])?;
					offset += 8 + new_child.length as usize;
					children.push(new_child);
				}

				(None, Some(children), Some(form_type))
			} else {
				let data = raw[8..8+(length as usize)].to_vec();

				(Some(data), None, None)
			}
		};


		Ok(RiffChunk {
			tag,
			length,
			data,
			children,
			form_type,
		})
	}

	pub fn child(&self, tag: &str) -> Option<&RiffChunk> {
		assert!(self.children.is_some());

		for child in self.children.as_ref().unwrap().iter() {
			if child.tag.as_str() == tag {
				return Some(child);
			}
		}

		None
	}
}

