use std::{collections::HashMap, fmt};

use image::io::Reader;

use crate::{etil, Error, Result, composite::CompositeImage};

/// an image lol
#[derive(Debug)]
pub enum Image {
	File,
	Bin(&'static [u8]),
}

impl Image {
	/// load an image into memory
	fn to_data(&self, path: &str) -> Result<CompositeImage> {
		Ok(match self {
			Self::File => CompositeImage::from_image(etil::cast_result(etil::cast_result(Reader::open(path), |e| Error::Io(e))?.decode(), |e| Error::Image(e))?.to_rgba8()),
			Self::Bin(data) => CompositeImage::from_image(etil::cast_result(image::load_from_memory(data), |e| Error::Image(e))?.to_rgba8()),
		})
	}
}

/// cache of images
pub struct ImageCache<'a> {
	images: HashMap<&'a str, Image>,
	loaded: HashMap<&'a str, CompositeImage>,
}

impl<'a> fmt::Debug for ImageCache<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("ImageCache")
			.field("images", &self.images)
			.finish()
	}
}

impl<'a> ImageCache<'a> {
	/// make new
	pub fn new() -> Self {
		Self {
			images: HashMap::new(),
			loaded: HashMap::new(),
		}
	}
	pub fn add(&mut self, id: &'a str, image: Image) {
		if self.images.insert(id.into(), image).is_some() {
			// panic: user error
			panic!("image already in cache");
		}
	}
	pub fn get(&self, id: &'a str) -> &CompositeImage {
		&self.loaded[id]
	}
	pub fn data(&mut self, id: &'a str) -> Result<()> {
		if !self.loaded.contains_key(id) {
			let data = self.images[id].to_data(id)?;
			self.loaded.insert(id, data);
		}
		Ok(())
	}
	pub fn get_data(&mut self, id: &'a str) -> Result<&CompositeImage> {
		if self.loaded.contains_key(id) {
			Ok(&self.loaded[id])
		} else {
			let data = self.images[id].to_data(id)?;
			self.loaded.insert(id, data);
			Ok(self.get(id))
		}
	}
	pub fn unload_all(&mut self) {
		self.loaded.clear();
	}
}
