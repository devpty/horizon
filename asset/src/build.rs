use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::common::{AssetError, AssetResult, Wad64Entry};

pub struct Builder {
	entries: Vec<Wad64Entry>,
	name_lump: Vec<String>,
	data_lump: Vec<Vec<u8>>,
	offset: u64,
}

const COMPRESSION_LEVEL: i32 = 19;

impl Builder {
	pub fn new() -> Self {
		Self {
			entries: vec![],
			name_lump: vec![],
			data_lump: vec![],
			offset: 16,
		}
	}
	pub fn build(self, out: impl AsRef<Path>) -> AssetResult<()> {
		let mut file = fs::File::create(out)?;
		file.write_all(b"qWAD64!!")?;
		file.write_all(&self.entries.len().to_be_bytes())?;
		let mut real_offset = self.offset;
		for ent in self.entries {
			file.write_all(&ent.name_len.to_be_bytes())?;
			file.write_all(&ent.data_len_high.to_be_bytes())?;
			file.write_all(&ent.data_len_low.to_be_bytes())?;
			file.write_all(&real_offset.to_be_bytes())?;
			real_offset += ((ent.data_len_high as u64) << 32) | ent.data_len_low as u64;
		}
		for name in self.name_lump {
			file.write_all(name.as_bytes())?;
		}
		for data in self.data_lump {
			file.write_all(&data)?;
		}
		Ok(())
	}
	fn bundle_append(&mut self, name: String, data: Vec<u8>) -> AssetResult<()> {
		let name_len = name.len();
		if name_len > 0xFFFF {
			return Err(AssetError::NameTooLong(name_len));
		}
		let name_len = name_len as u16;
		let data_len = data.len();
		if data_len > 0xFFFFFFFFFFFF {
			return Err(AssetError::DataTooLong(data_len));
		}
		let data_len = data_len as u64;
		let data_len_high = (data_len >> 32) as u16;
		let data_len_low = (data_len & 0xFFFFFFFF) as u32;
		self.entries.push(Wad64Entry {
			name_len,
			data_len_high,
			data_len_low,
			data_ptr: 0,
		});
		self.name_lump.push(name);
		self.data_lump.push(data);
		self.offset += 16 + name_len as u64;
		Ok(())
	}
	pub fn bundle_data(
		&mut self,
		dest: impl Into<String>,
		source: impl AsRef<[u8]>,
	) -> AssetResult<()> {
		let mut output = io::Cursor::new(vec![]);
		let mut input = io::Cursor::new(source.as_ref().to_vec());
		zstd::stream::copy_encode(&mut input, &mut output, COMPRESSION_LEVEL)?;
		// io::copy(&mut input, &mut output)?;
		self.bundle_append(dest.into(), output.into_inner())
	}
	pub fn bundle_path(
		&mut self,
		dest: impl AsRef<str>,
		source: impl AsRef<Path>,
	) -> AssetResult<()> {
		for (source, dest) in recurse_dir(source.as_ref(), dest.as_ref())? {
			let mut output = io::Cursor::new(vec![]);
			let mut input = fs::File::open(source)?;
			zstd::stream::copy_encode(&mut input, &mut output, COMPRESSION_LEVEL)?;
			// io::copy(&mut input, &mut output)?;
			self.bundle_append(dest, output.into_inner())?;
		}
		Ok(())
	}
	pub fn ctdata_data<A, B>(&mut self, dest: impl AsRef<Path>, source: impl AsRef<[u8]>) {}
}

impl Default for Builder {
	fn default() -> Self {
		Self::new()
	}
}

fn recurse_dir(dir: &Path, map_res: &str) -> AssetResult<Vec<(PathBuf, String)>> {
	let mut res = vec![];
	if dir.is_dir() {
		for entry in fs::read_dir(dir)? {
			let path = entry?.file_name();
			let mut path_dir = dir.to_path_buf();
			path_dir.push(&path);
			let mut path_map = map_res.to_string();
			path_map.push('/');
			path_map.push_str(&path.to_string_lossy());
			res.append(&mut recurse_dir(path_dir.as_path(), &path_map)?);
		}
	} else {
		res.push((dir.to_path_buf(), map_res.to_string()));
	}
	Ok(res)
}
