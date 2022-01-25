use std::{collections, fmt};
// use std::hash::{Hash, Hasher};
use super::rectpack2d;

#[derive(Copy, Clone)]
pub enum PackerImage {
	Unique {
		image: crate::Image,
		source_loc: (u32, u32),
		packed_pos: crate::Rect,
	},
	// index in vector
	Duplicate(PackerKey, usize),
}

impl fmt::Debug for PackerImage {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Unique {
				image, source_loc, packed_pos
			} => {
				f.write_fmt(format_args!("({:?}, {:?}, {:?})", image, source_loc, packed_pos))
			},
			Self::Duplicate(key, idx) => {
				f.write_fmt(format_args!("(ref {:?}:{})", key, idx))
			}
		}
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

#[derive(Debug)]
pub struct Packer {
	pub allow_flipping: bool,
	images: collections::HashMap<PackerKey, Vec<PackerImage>>,
}

impl Packer {
	pub fn new() -> Self {
		Self {
			allow_flipping: false,
			images: collections::HashMap::new()
		}
	}
	pub fn allow_flipping(&mut self, v: bool) {
		self.allow_flipping = v;
	}
	pub fn add_image(&mut self, id: &'static str, layer: Option<&'static str>, cache: &mut crate::ImageCache, image: crate::Image, ty: crate::ImageType) {
		self.images.insert(
			PackerKey {id, layer},
			ty.to_rects(&image, cache).iter().map(|v| PackerImage::Unique {image, source_loc: (v.0, v.1), packed_pos: crate::Rect(0, 0, v.2, v.3, v.4)}).collect()
		);
	}
	pub fn dedup(&mut self, cache: &mut crate::ImageCache) {
		let mut set = collections::HashMap::<Vec<u8>, (PackerKey, usize)>::new();
		for i in self.images.iter_mut() {
			println!("checking {:?}", i.0);
			for j in i.1.iter_mut().enumerate() {
				print!("- image {:>3} ({:?}): ", j.0, j.1);
				match j.1 {
					PackerImage::Unique {
						image, packed_pos, source_loc
					} => {
						let bytes = image.to_entry(cache).crop(packed_pos.with_pos(*source_loc)).into_raw();
						if set.contains_key(&bytes) {
							let dup = set.get(&bytes).unwrap();
							println!("duplicate {:?}:{}", dup.0, dup.1);
							// mark it as a duplicate
							*j.1 = PackerImage::Duplicate(dup.0, dup.1);
						} else {
							println!("unique {}", j.0);
							set.insert(bytes, (*i.0, j.0));
						}
					},
					PackerImage::Duplicate(key, idx) => {
						println!("pre-duplicate {:?}:{}", key, idx);
					},
				}
			}
		}
	}
	pub fn pack(&mut self, cache: &mut crate::ImageCache) {
		let mut rects_by_layer = collections::HashMap::<Option<&str>, collections::HashMap<&str, &Vec<PackerImage>>>::new();
		for (i, iv) in &self.images {
			// id's can be duplicated between layers
			let entry = rects_by_layer.entry(i.layer).or_default();
			// todo: all rects of the same id & index should have the same resolution
			// but they're unique within layers
			if entry.insert(i.id, iv).is_some() {
				// panic: user error, not my problem
				panic!("duplicate identifier {}", i.id);
			}
		}
		// iterate through atlases and generate
		for (i, iv) in rects_by_layer {
			let mut rects_to_place = Vec::new();
			let mut rects_info = Vec::new();
			for (j, jv) in iv {
				for (k, kv) in jv.iter().enumerate() {
					// todo: image size equality checks
					match kv {
						PackerImage::Unique {
							image, source_loc, packed_pos
						} => {
							let rect = rectpack2d::rect_structs::RectXYWHF::from(0, 0, packed_pos.2, packed_pos.3, false);
							// rotation isn't yet supported :(
							// rect.allow_global_z_axis_rotation = true;
							rects_to_place.push(rect);
							rects_info.push((j, k))
						},
						// don't handle duplicates as their originals are already handled
						PackerImage::Duplicate(..) => {},
					}
				}
			}
			let rect_size = rectpack2d::finders_interface::find_best_packing(
				&mut rects_to_place,
				16384,
				rectpack2d::finders_interface::DiscardStep::Tries(4),
				true,
				rectpack2d::finders_interface::DEFAULT_COMPARATORS,
			);
			println!("{:?}: {:#?} {:#?}", i, rect_placements, target_bins[&()]);
		}
	}
}
