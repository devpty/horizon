use std::{collections::HashMap};

use crate::{ImageCache, Image, Result, rectpack2d::{RectXYWH, RectWH, self}, Error, composite::{CompositeImage, rect_idx}};

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
	bin_size: HashMap<Option<&'a str>, RectWH>,
}

impl<'a> Packer<'a> {
	pub fn new(cache: &'a mut ImageCache<'a>) -> Self {
		Self { cache, images: HashMap::new(), bin_size: HashMap::new() }
	}
	pub fn rects_of(&mut self, path: &'a str, data: ImageLoad) -> Result<Vec<RectXYWH>> {
		match data {
			ImageLoad::Whole => Ok(vec![self.cache.get_data(path)?.size.to_xywh()]),
			ImageLoad::Tiled {
				init, gap, count
			} => Ok(rect_idx(count).iter().map(|r| RectXYWH::new(
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
				if let ImageData::Unique { key, .. } = jv {
					self.cache.data(key)?
				}
			}
		}
		Ok(())
	}
	/// loads *every* image
	pub fn deduplicate(&mut self) -> Result<()> {
		let mut image_layer_map = HashMap::new();
		self.load_all()?;
		for (ik, iv) in self.images.iter_mut() {
			let image_map = image_layer_map.entry(ik.1).or_insert_with(|| HashMap::new());
			for (jk, jv) in iv.iter_mut().enumerate() {
				match jv {
					ImageData::Unique { key, source, .. } => {
						let full_data = self.cache.get(key);
						let mut data = CompositeImage::new(source.to_wh());
						data.copy_from(full_data, *source, RectWH::new(0, 0));
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
		self.bin_size.clear();
		let mut rects_by_layer = HashMap::new();
		for (ik, iv) in self.images.iter_mut() {
			rects_by_layer.entry(ik.1)
				.or_insert_with(|| HashMap::new())
				.insert(ik.0, iv);
		}
		let mut rects_output = HashMap::new();
		for (ik, iv) in rects_by_layer {
			let mut rects = Vec::new();
			let mut rect_info = Vec::new();
			for (jk, jv) in iv {
				for (kk, kv) in jv.iter().enumerate() {
					if let ImageData::Unique { source, .. } = kv {
						rects.push(source.reset_xy());
						rect_info.push((jk, kk));
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
			self.bin_size.insert(ik, bin_size);
			rects_output.insert(ik, (rects, rect_info));
		}
		for (ik, (av, ai)) in rects_output {
			for (rv, (jk, rk)) in av.iter().zip(ai.iter()) {
				let data = &mut self.images.get_mut(&ImageKey(jk, ik)).unwrap()[*rk];
				if let ImageData::Unique { key, source, .. } = data {
					*data = ImageData::Unique {
						key: *key,
						source: *source,
						dest: RectWH::new(rv.x, rv.y),
					}
				}
			}
		}
		Ok(())
	}
	fn unique_entry(&self, v: &ImageData) -> UniqueImageData {
		match v {
			ImageData::Unique { key, source, dest } => UniqueImageData {
				key: key.to_string(),
				source: *source,
				dest: *dest
			},
			ImageData::Ref(key, idx) => self.unique_entry(&self.images.get(key).unwrap()[*idx]),
		}
	}
	pub fn export(&self) -> HashMap<Option<String>, HashMap<String, Vec<RectXYWH>>> {
		let mut out = HashMap::new();
		for (ik, iv) in &self.images {
			out.entry(match ik.1 {
				Some(v) => Some(v.to_string()),
				None => None,
			}).or_insert_with(|| HashMap::new())
				.insert(ik.0.to_string(), iv.iter().map(|v| {
				let ent = self.unique_entry(v);
				RectXYWH::new(ent.dest.w, ent.dest.h, ent.source.w, ent.source.h)
			}).collect::<Vec<_>>());
		}
		out
	}
	pub fn composite(&mut self) -> Result<HashMap<Option<String>, CompositeImage>> {
		self.load_all()?;
		let mut out = HashMap::new();
		let mut rects_by_layer = HashMap::new();
		for (ik, iv) in &self.images {
			rects_by_layer.entry(ik.1)
				.or_insert_with(|| HashMap::new())
				.insert(ik.0, iv);
		}
		for (ik, iv) in rects_by_layer {
			let mut image = CompositeImage::new(self.bin_size[&ik]);
			for (_jk, jv) in iv {
				for kv in jv {
					let ent = self.unique_entry(kv);
					image.copy_from(self.cache.get(&ent.key), ent.source, ent.dest);
				}
			}
			out.insert(match ik {
				Some(v) => Some(v.to_string()),
				None => None,
			}, image);
		}
		Ok(out)
	}
}

#[derive(Debug, Clone)]
struct UniqueImageData {
	key: String,
	source: RectXYWH,
	dest: RectWH,
}
