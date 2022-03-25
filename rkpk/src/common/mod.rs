use std::fmt;
use thiserror::Error;

pub type ImagePos = u16;
pub type ImageSize = (ImagePos, ImagePos);
pub type ImageRect = (ImagePos, ImagePos, ImagePos, ImagePos);

#[derive(Error, Debug)]
pub enum RkPkError {
	#[error("io error")]
	IoError(#[from] std::io::Error),
	#[error("image error")]
	ImageError(#[from] image::ImageError),
}

pub type RkPkResult<T> = Result<T, RkPkError>;

/// composite image
pub struct CompositeImage {
	pub size: ImageSize,
	pub data: Vec<u8>,
}

impl fmt::Debug for CompositeImage {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("CompositeImage")
			.field("data", &self.data.len())
			.field("size", &self.size)
			.finish()
	}
}

/// utility function
pub fn rect_idx(r: ImageSize) -> Vec<ImageSize> {
	(0..r.0 * r.1).map(|v| (v % r.0, v / r.1)).collect()
}

impl CompositeImage {
	/// new one with size
	pub fn new(size: ImageSize) -> Self {
		let area = size.0 as usize * size.1 as usize;
		let mut data = vec![0xFFu8; area * 4];
		for i in 0..area {
			data[i * 4 + 1] = 0;
		}
		Self { data, size }
	}
	// copy from another image
	pub fn copy_from(
		&mut self,
		other: &CompositeImage,
		self_offset: ImageSize,
		other_uv: ImageRect,
	) {
		// bounds checking
		if other_uv.0 + other_uv.2 > other.size.0
			|| other_uv.1 + other_uv.3 > other.size.1
			|| self_offset.0 + other_uv.2 > self.size.0
			|| self_offset.1 + other_uv.3 > self.size.1
		{
			panic!("out of bounds copy");
		}
		let step = other_uv.2 as usize * 4;
		if self.size.0 == other.size.0 && other_uv.2 == self.size.0 {
			// direct copy
			let s_start = (other_uv.1 * other_uv.2) as usize * 4;
			let d_start = (self_offset.1 * other_uv.2) as usize * 4;
			let step = step * other_uv.3 as usize;
			self.data[d_start..d_start + step]
				.copy_from_slice(&other.data[s_start..s_start + step]);
		} else {
			for y in 0..other_uv.3 {
				let s_start = ((y + other_uv.1) * other.size.0 + other_uv.0) as usize * 4;
				let d_start = ((y + self_offset.1) * self.size.0 + self_offset.0) as usize * 4;
				self.data[d_start..d_start + step]
					.copy_from_slice(&other.data[s_start..s_start + step]);
			}
		}
	}
}

impl From<image::RgbaImage> for CompositeImage {
	fn from(image: image::RgbaImage) -> Self {
		let size = (image.width() as ImagePos, image.height() as ImagePos);
		Self {
			size,
			data: image.into_raw(),
		}
	}
}

impl From<CompositeImage> for image::RgbaImage {
	fn from(image: CompositeImage) -> Self {
		Self::from_raw(image.size.0 as u32, image.size.1 as u32, image.data).unwrap()
	}
}
