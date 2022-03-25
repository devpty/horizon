use std::{fmt, path::Path};

use image::RgbaImage;

use crate::{rectpack2d::{RectWH, RectXYWH}, Error, Result};

/// image to copy things between
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompositeImage {
	pub data: Vec<u8>,
	pub size: RectWH,
}

impl fmt::Debug for CompositeImage {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("CompositeImage").field("data", &self.data.len()).field("size", &self.size).finish()
	}
}

/// utility function
pub fn rect_idx(r: RectWH) -> Vec<RectWH> {
	(0..r.w*r.h).map(|v| RectWH::new(v % r.w, v / r.w)).collect()
}

impl CompositeImage {
	/// new one with size
	pub fn new(size: RectWH) -> Self {
		let area = size.area() as usize;
		let mut data = vec![0xFFu8; area * 4];
		for i in 0..area { data[i * 4 + 1] = 0; }
		Self {
			data,
			size
		}
	}
	/// load a template image
	pub fn from_image(image: RgbaImage) -> Self {
		let size = RectWH::new(image.width(), image.height());
		Self { size, data: image.into_raw() }
	}
	// copy from another image
	pub fn copy_from(&mut self, other: &CompositeImage, source: RectXYWH, dest: RectWH) {
		let step = source.w as usize * 4;
		for y in 0..source.h {
			let s_start = ((y + source.y) * other.size.w + source.x) as usize * 4;
			let d_start = ((y + dest.h) * self.size.w + dest.w) as usize * 4;
			self.data[d_start..d_start + step].copy_from_slice(&other.data[s_start..s_start + step]);
		}
	}
	/// save the image to disk
	pub fn save_to_disk(&self, path: &Path, fmt: image::ImageFormat) -> Result<()> {
		image::save_buffer_with_format(
			path, &self.data, self.size.w, self.size.h, image::ColorType::Rgba8, fmt
		).map_err(Error::Image)
	}
}
