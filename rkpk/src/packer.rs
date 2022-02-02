use std::{collections::HashMap};

use crate::{ImageCache, Image, cache::size_of_image, Result, rectpack2d::{self, RectXYWH, RectWH}, Error};

pub enum ImageLoad {
	Whole,
	Tiled {
		init: RectXYWH,
		gap: RectWH,
		count: RectWH,
	},
	Atlas(Vec<RectXYWH>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ImageKey<'a>(&'a str, Option<&'a str>);
#[derive(Debug, Clone)]
enum ImageData<'a> {
	Unique {
		key: &'a str,
		source: RectXYWH,
		dest: RectWH,
	},
	Ref(ImageKey<'a>, usize),
}

#[derive(Debug)]
pub struct Packer<'a> {
	cache: &'a mut ImageCache<'a>,
	images: HashMap<ImageKey<'a>, Vec<ImageData<'a>>>,
	bin_size: RectWH,
}

fn rect2d(r: RectWH) -> Vec<RectWH> {
	(0..r.w*r.h).map(|v| RectWH::new(v % r.w, v / r.w)).collect()
}

impl<'a> Packer<'a> {
	pub fn new(cache: &'a mut ImageCache<'a>) -> Self {
		Self { cache, images: HashMap::new(), bin_size: RectWH::new(0, 0) }
	}
	pub fn rects_of(&mut self, path: &'a str, data: ImageLoad) -> Result<Vec<RectXYWH>> {
		match data {
			ImageLoad::Whole => {
				let data = size_of_image(self.cache.get_data(path)?);
				Ok(vec![RectXYWH::new(0, 0, data.0, data.1)])
			},
			ImageLoad::Tiled {
				init, gap, count
			} => Ok(rect2d(count).iter().map(|r| RectXYWH::new(
				init.x + gap.w + (gap.w + init.w) * r.w,
				init.y + gap.h + (gap.h + init.h) * r.h,
				init.w, init.h,
			)).collect()),
			ImageLoad::Atlas(data) => Ok(data)
		}
	}
	pub fn add(&mut self, id: &'a str, group: Option<&'a str>, path: &'a str, load: ImageLoad) -> Result<()> {
		let rects = self.rects_of(&path, load)?.iter().map(|rect| ImageData::Unique {
			key: path,
			source: *rect,
			dest: RectWH::new(0, 0),
		}).collect();
		self.images.insert(ImageKey(id, group), rects);
		Ok(())
	}
	pub fn add_new(&mut self, id: &'a str, group: Option<&'a str>, path: &'a str, ty: Image, load: ImageLoad) -> Result<()> {
		self.cache.add(path, ty);
		self.add(id, group, path, load)
	}
	fn load_all(&mut self) -> Result<()> {
		for (_, iv) in &self.images {
			for jv in iv {
				match jv {
					ImageData::Unique { key, .. } => self.cache.data(key)?,
					_ => {},
				}
			}
		}
		Ok(())
	}
	/// loads *every* image
	pub fn deduplicate(&mut self) -> Result<()> {
		let mut image_map = HashMap::new();
		self.load_all()?;
		for (ik, iv) in self.images.iter_mut() {
			for (jk, jv) in iv.iter_mut().enumerate() {
				match jv {
					ImageData::Unique { key, source, .. } => {
						let full_data = self.cache.get(key);
						let mut data = CompositeImage::new(source.w, source.h);
						data.copy_from(full_data, source.x, source.y, source.w, source.h, 0, 0);
						match image_map.get(&data) {
							Some((nik, njk)) => *jv = ImageData::Ref(*nik, *njk),
							None => {image_map.insert(data, (*ik, jk));},
						}
					}, _ => {},
				}
			}
		}
		Ok(())
	}
	pub fn pack(&mut self) -> Result<()> {
		// <layer, <name, <id, image>>>
		let mut rects_by_layer = HashMap::new();
		for (ik, iv) in &self.images {
			let rects_by_name = rects_by_layer.entry(ik.1).or_insert_with(|| HashMap::new());
			rects_by_name.insert(ik.0, iv);
		}
		for (ik, iv) in rects_by_layer {
			let mut rects = Vec::new();
			let mut rect_info = Vec::new();
			for (jk, jv) in iv {
				for (kk, kv) in jv.iter().enumerate() {
					match kv {
						ImageData::Unique { source, .. } => {
							rects.push(source.clone());
							rect_info.push((jk, kk));
						},
						_ => {},
					}
				}
			}
			let bin_size = rectpack2d::find_best_packing(
				&mut rects, 16384,
				rectpack2d::DiscardStep::Tries(4),
				rectpack2d::DEFAULT_COMPARATORS
			);
			let bin_size = match bin_size {
				Some(v) => v,
				None => return Err(Error::PackingFailed),
			};
		}
		Ok(())
	}
}
