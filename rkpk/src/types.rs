use std::{collections, fmt};
// use std::io;

/// rectangle with flipping
#[derive(Debug, Copy, Clone)]
pub struct Rect(pub u32, pub u32, pub u32, pub u32, pub bool);

#[derive(Copy, Clone)]
pub struct PackerImage {
	pub image: Image,
	pub source_loc: (u32, u32),
	pub packed_pos: Rect,
}

impl fmt::Debug for PackerImage {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_fmt(format_args!("({:?}, {:?}, {:?})", self.image, self.source_loc, self.packed_pos))
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PackerKey {
	pub id: &'static str,
	pub layer: Option<&'static str>,
}

impl fmt::Debug for PackerKey {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_fmt(format_args!("{:?}:{:?}", self.layer, self.id))
	}
}

#[derive(Clone)]
pub struct ImageCacheEntry {
	pub data: image::RgbaImage,
}

impl fmt::Debug for ImageCacheEntry {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_fmt(format_args!("ImageCacheEntry({}×{})", self.data.width(), self.data.height()))
	}
	fn crop(r: Rect) {

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
			self.0.insert(path, ImageCacheEntry {
				data: image.to_data()
			});
		}
		self.0.get(path).unwrap()
	}
}

#[derive(Debug, Copy, Clone)]
pub enum Image {
	External(&'static str),
}

impl Image {
	pub fn to_path(&self) -> &'static str {
		match self {
			Self::External(path) => path,
		}
	}
	pub fn to_entry<'a>(&self, cache: &'a mut ImageCache) -> &'a ImageCacheEntry {
		cache.get_fill(self.to_path(), &self)
	}
	pub fn to_data(&self) -> image::RgbaImage {
		match self {
			Self::External(path) =>
				image::io::Reader::open(path)
					.unwrap()
					.decode()
					.unwrap()
					.to_rgba8(),
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub enum ImageType {
	/// use the entire image
	Whole,
	/// crop a rectangluar region
	Crop(Rect),
	/// treat image as a list of images using a tileset
	///
	/// props: start_pos, tile_size, gap_size, tile_count
	Tiled((u32, u32), (u32, u32), (u32, u32), (u32, u32)),
}

impl ImageType {
	pub fn to_rects(&self, image: &Image, cache: &mut ImageCache) -> Vec<Rect> {
		match self {
			Self::Whole => {
				let entry = image.to_entry(cache);
				vec![Rect(0, 0, entry.data.width(), entry.data.height(), false)]
			}
			Self::Crop(r) => vec![*r],
			Self::Tiled(start_pos, tile_size, gap_size, tile_count) => {
				let (start_pos_x, start_pos_y) = *start_pos;
				let (tile_size_x, tile_size_y) = *tile_size;
				let (gap_size_x, gap_size_y) = *gap_size;
				let (tile_count_x, tile_count_y) = *tile_count;
				let mut out = vec![Rect(0, 0, 0, 0, false); (tile_count_x * tile_count_y).try_into().unwrap()];
				for x in 0..tile_count_x {
					for y in 0..tile_count_y {
						out[(x + y * tile_count_x) as usize] = Rect(
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
