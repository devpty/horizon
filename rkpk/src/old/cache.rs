use std::{collections, fmt};
use crate::Rect;

#[derive(Clone)]
pub struct ImageCacheEntry {
	pub data: image::RgbaImage,
}

impl ImageCacheEntry {
	pub fn crop(&self, r: Rect) -> image::RgbaImage {
		// we've determined that flip means "this rect is rotated"
		// for now, we ignore rotation
		let src = self.data.as_ref();
		let mut dst = vec![0u8; r.2 as usize * r.3 as usize * 4];
		let dst_step = r.2 as usize * 4;
		// println!("crop dst_step={}", dst_step);
		for line_y in 0..r.3 {
			let dst_start = dst_step * line_y as usize;
			let src_start = (r.0 + (r.1 + line_y) * self.data.width()) as usize * 4;
			// println!("crop line_y={}, dst_start={}, src_start={}", line_y, dst_start, src_start);
			dst[dst_start..dst_start + dst_step].clone_from_slice(&src[src_start..src_start + dst_step]);
		}
		// unwrap: can't fail since the buffer is the same size
		image::RgbaImage::from_vec(r.2, r.3, dst).unwrap()
	}
}

impl fmt::Debug for ImageCacheEntry {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_fmt(format_args!("ImageCacheEntry({}×{})", self.data.width(), self.data.height()))
	}
}

#[derive(Debug, Clone)]
pub struct ImageCache(collections::HashMap<&'static str, ImageCacheEntry>);

impl ImageCache {
	pub fn new() -> Self {
		Self(collections::HashMap::new())
	}
	fn get_fill(&mut self, path: &'static str, image: &Image) -> &ImageCacheEntry {
		if !self.0.contains_key(path) {
			println!("loading {:?}", image);
			self.0.insert(path, ImageCacheEntry { // < here
				data: image.to_data()
			});
		}
		// unwrap: can't fail since it was inserted ^
		self.0.get(path).unwrap()
	}
}

#[derive(Debug, Copy, Clone)]
pub enum Image {
	External(&'static str),
	Bytes(&'static [u8]),
}

impl Image {
	pub fn to_path(&self) -> &'static str {
		match self {
			Self::External(path) => path,
			Self::Bytes(_) => "[byte-array]"
		}
	}
	pub fn to_entry<'a>(&self, cache: &'a mut ImageCache) -> &'a ImageCacheEntry {
		cache.get_fill(self.to_path(), &self)
	}
	pub fn to_data(&self) -> image::RgbaImage {
		match self {
			Self::External(path) =>
				image::io::Reader::open(path)
					.unwrap() // unwrap-fail: can fail if file not found
					.decode()
					.unwrap() // unwrap-fail: can fail if image isn't valid
					.to_rgba8(),
			Self::Bytes(data) =>
				image::load_from_memory(data)
					.unwrap() // unwrap-fail: can fail if image isn't valid
					.to_rgba8()
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub enum ImageType {
	/// use the entire image
	Whole,
	/// crop a rectangluar region
	Crop(crate::Rect),
	/// treat image as a list of images using a tileset
	///
	/// props: start_pos, tile_size, gap_size, tile_count
	Tiled((u32, u32), (u32, u32), (u32, u32), (u32, u32)),
}

impl ImageType {
	pub fn to_rects(&self, image: &Image, cache: &mut ImageCache) -> Vec<crate::Rect> {
		match self {
			Self::Whole => {
				let entry = image.to_entry(cache);
				vec![crate::Rect(0, 0, entry.data.width(), entry.data.height(), false)]
			}
			Self::Crop(r) => vec![*r],
			Self::Tiled(start_pos, tile_size, gap_size, tile_count) => {
				let (start_pos_x, start_pos_y) = *start_pos;
				let (tile_size_x, tile_size_y) = *tile_size;
				let (gap_size_x, gap_size_y) = *gap_size;
				let (tile_count_x, tile_count_y) = *tile_count;
				let mut out = vec![crate::Rect(0, 0, 0, 0, false); tile_count_x as usize * tile_count_y as usize];
				for x in 0..tile_count_x {
					for y in 0..tile_count_y {
						out[(x + y * tile_count_x) as usize] = crate::Rect(
							start_pos_x + x * (tile_size_x + gap_size_x),
							start_pos_y + y * (tile_size_y + gap_size_y),
							tile_size_x, tile_size_y,
							false
						);
					}
				}
				out
			}
		}
	}
}
