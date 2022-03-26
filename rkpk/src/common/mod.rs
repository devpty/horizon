use std::fmt;
use thiserror::Error;

pub type ImagePos = u16;
pub type ImageArea = u32;

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
	pub size: RectWH,
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

impl CompositeImage {
	/// new one with size
	pub fn new(size: RectWH) -> Self {
		let area = size.w as usize * size.h as usize;
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
		self_offset: RectWH,
		other_uv: RectXYWH,
	) {
		// bounds checking
		if other_uv.x + other_uv.w > other.size.w
			|| other_uv.y + other_uv.h > other.size.h
			|| self_offset.w + other_uv.w > self.size.w
			|| self_offset.h + other_uv.h > self.size.h
		{
			panic!("out of bounds copy");
		}
		let step = other_uv.w as usize * 4;
		if self.size.w == other.size.w && other_uv.w == self.size.w {
			// direct copy
			let s_start = (other_uv.y * other_uv.w) as usize * 4;
			let d_start = (self_offset.h * other_uv.w) as usize * 4;
			let step = step * other_uv.h as usize;
			self.data[d_start..d_start + step]
				.copy_from_slice(&other.data[s_start..s_start + step]);
		} else {
			for y in 0..other_uv.h {
				let s_start = ((y + other_uv.y) * other.size.w + other_uv.x) as usize * 4;
				let d_start = ((y + self_offset.h) * self.size.w + self_offset.w) as usize * 4;
				self.data[d_start..d_start + step]
					.copy_from_slice(&other.data[s_start..s_start + step]);
			}
		}
	}
}

impl From<image::RgbaImage> for CompositeImage {
	fn from(image: image::RgbaImage) -> Self {
		Self {
			size: RectWH::new(image.width() as ImagePos, image.height() as ImagePos),
			data: image.into_raw(),
		}
	}
}

impl From<CompositeImage> for image::RgbaImage {
	fn from(image: CompositeImage) -> Self {
		Self::from_raw(image.size.w as u32, image.size.h as u32, image.data).unwrap()
	}
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RectWH {
	pub w: ImagePos,
	pub h: ImagePos,
}

impl RectWH {
	pub fn new(w: ImagePos, h: ImagePos) -> Self {
		Self { w, h }
	}
	pub fn max_size(&self) -> ImagePos {
		if self.h > self.w {
			self.h
		} else {
			self.w
		}
	}
	pub fn min_size(&self) -> ImagePos {
		if self.h < self.w {
			self.h
		} else {
			self.w
		}
	}
	pub fn area(&self) -> ImageArea {
		self.w as ImageArea * self.h as ImageArea
	}
	pub fn perimeter(&self) -> ImageArea {
		2 * (self.w as ImageArea + self.h as ImageArea)
	}
	pub fn path_mul(&self) -> f64 {
		self.max_size() as f64 / (self.min_size() as ImageArea * self.area()) as f64
	}
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RectXYWH {
	pub x: ImagePos,
	pub y: ImagePos,
	pub w: ImagePos,
	pub h: ImagePos,
}

impl RectXYWH {
	pub fn new(x: ImagePos, y: ImagePos, w: ImagePos, h: ImagePos) -> Self {
		Self { x, y, w, h }
	}
	pub fn area(&self) -> ImageArea {
		self.w as ImageArea * self.h as ImageArea
	}
	pub fn to_wh(&self) -> RectWH {
		RectWH::new(self.w, self.h)
	}
}
