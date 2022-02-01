use std::collections::HashMap;

use image::{RgbaImage, io::Reader};

use crate::{etil, Error};

/// an image lol
pub enum Image {
	File(String),
	Bin(&'static [u8]),
}

impl Image {
	/// load an image into memory
	fn to_data(&self) -> Result<RgbaImage, Error> {
		Ok(match self {
			Self::File(path) => etil::cast_result(etil::cast_result(Reader::open(path), |e| Error::Io(e))?.decode(), |e| Error::Image(e))?.to_rgba8(),
			Self::Bin(data) => etil::cast_result(image::load_from_memory(data), |e| Error::Image(e))?.to_rgba8()
		})
	}
}

enum CacheImage {
	Unloaded(Image),
	Loaded(RgbaImage),
}

/// cache of images
pub struct ImageCache {
	images: HashMap<String, CacheImage>,
}

impl ImageCache {
	/// make new
	fn new() -> Self {
		Self {
			images: HashMap::new(),
		}
	}
	fn add(&mut self, id: String, image: Image) {
		if self.images.insert(id, CacheImage::Unloaded(image)).is_some() {
			// panic: user error
			panic!("image already in cache");
		}
	}
}
