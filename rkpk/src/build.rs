//! utilities for using rkpk for build scripts

use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::fs;
use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::collections::HashMap;

use crate::Image;
use crate::ImageCache;
use crate::ImageLoad;
use crate::Packer;
use crate::rectpack2d::RectWH;
use crate::rectpack2d::RectXYWH;

macro_rules! bprintln {
	($($arg:tt)*) => {{
		let text = format!($($arg)*);
		for line in text.lines() {
			println!("cargo:warning=\x1b[G\x1b[K{}", line);
		}
	}}
}

#[derive(Debug)]
enum InError {
	Io(io::Error),
}

type InResult<T> = Result<T, InError>;


fn visit_dir(dir: &Path) -> InResult<Vec<PathBuf>> {
	let mut res = vec![];
	if dir.is_dir() {
		for entry in fs::read_dir(dir).map_err(InError::Io)? {
			let path = entry.map_err(InError::Io)?.path();
			res.append(&mut visit_dir(path.as_path())?);
		}
	} else {
		res.push(dir.to_path_buf());
	}
	Ok(res)
}

#[derive(serde::Deserialize)]
struct ImageMetadata {
	start: (u32, u32),
	size: (u32, u32),
	gap: (u32, u32),
	count: (u32, u32),
}

pub fn auto_make(dir: &Path) {
	let out_path = env::var_os("OUT_DIR").unwrap();
	let out_dir = Path::new(&out_path);
	let mut inputs = vec![];
	let mut outputs = HashMap::new();
	for file in visit_dir(dir).expect("Failed to scan files") {
		let ext = file.extension();
		let ext_str = ext.unwrap_or(OsStr::new("")).to_string_lossy();
		match ext_str.as_ref() {
			"png" => {
				// graph/[layer]/[file].png -> out/graph/[layer].png

				let mut target = out_dir.join(file
					.strip_prefix("src/")
					.unwrap());
				let image = target.file_name().unwrap().to_owned();
				let atlas = target.parent().unwrap().to_owned();
				target.pop();
				let mut atlas_str: OsString = atlas.as_os_str().into();
				atlas_str.push(".png");
				target.push(Path::new(&atlas_str));
				bprintln!("{:?} -> {:?}", file, target);
				let layer = atlas.as_os_str().to_string_lossy().into_owned();
				let layer = if layer == "global" {None} else {Some(layer)};
				inputs.push((image, layer, file.to_owned()));
				outputs.insert(layer, target);
			},
			_ => {},
		}
	}
	let mut cache = ImageCache::new();
	let mut packer = Packer::new(&mut cache);
	for (name, layer, path) in inputs {
		let	metadata = {
			let mut atlas_path = path.clone();
			add_extension(&mut atlas_path, "atlas");
			if atlas_path.exists() {
				ImageLoad::Atlas()
			} else {
				let mut data_path = path.clone();
				add_extension(&mut data_path, "toml");
				if data_path.exists() {
					let decode: ImageMetadata = toml::from_slice(&fs::read(data_path).unwrap()).unwrap();
					ImageLoad::Tiled {
						init: RectXYWH::new(decode.start.0, decode.start.1, decode.size.0, decode.size.1),
						gap: RectWH::new(decode.gap.0, decode.gap.1),
						count: RectWH::new(decode.count.0, decode.count.1),
					}
				} else {
					ImageLoad::Whole
				}
			}
		};
		packer.add_new(
			&name.as_os_str().to_string_lossy(),
			match layer {
				Some(v) => Some(&v),
				None => None
			},
			&path.as_os_str().to_string_lossy(),
			Image::File,
			metadata
		);
	}
	// let dest_path = Path::new(&out_dir).join("hello.rs");
	// fs::write(
	//     &dest_path,
	//     "pub fn message() -> &'static str {
	//         \"Hello, World!\"
	//     }
	//     "
	// ).unwrap();
}

fn add_extension(path: &mut PathBuf, ext: &str) {
	let new_ext = match path.extension().map(|v| v.to_string_lossy().into_owned()) {
		Some(v) => { v.push('.'); v },
		None => "".to_string(),
	};
	new_ext.push_str(ext);
	path.set_extension(new_ext);
}
