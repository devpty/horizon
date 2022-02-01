use std::collections::HashMap;

use image::{RgbaImage, io::Reader};

use crate::{etil, Error, Result};

/// an image lol
pub enum Image {
	File(String),
	Bin(&'static [u8]),
}

impl Image {
	/// load an image into memory
	fn to_data(&self) -> Result<RgbaImage> {
		Ok(match self {
			Self::File(path) => etil::cast_result(etil::cast_result(Reader::open(path), |e| Error::Io(e))?.decode(), |e| Error::Image(e))?.to_rgba8(),
			Self::Bin(data) => etil::cast_result(image::load_from_memory(data), |e| Error::Image(e))?.to_rgba8()
		})
	}
}

/// cache of images
pub struct ImageCache {
	images: HashMap<String, Image>,
	loaded: HashMap<String, RgbaImage>
}

enum ImageCacheTemp<'a> {
	Loaded(&'a RgbaImage),
	Unloaded(RgbaImage),
}

impl ImageCache {
	/// make new
	fn new() -> Self {
		Self {
			images: HashMap::new(),
			loaded: HashMap::new(),
		}
	}
	fn add(&mut self, id: String, image: Image) {
		if self.images.insert(id, image).is_some() {
			// panic: user error
			panic!("image already in cache");
		}
	}
	fn get_data<'a>(&'a mut self, id: String) -> Result<&'a RgbaImage> {
		if self.loaded.contains_key(&id) {
			Ok(&self.loaded[&id])
		} else if self.images.contains_key(&id) {
			// todo here
			Err(Error::Shit)
		} else {
			Err(Error::Shit)
		}
	}
}
